//! PostgreSQL wire protocol (version 3) over a plain TCP socket.
//!
//! Supported: a TLS upgrade (rustls), startup, cleartext and SCRAM-SHA-256
//! authentication, the simple query protocol and the extended query protocol
//! with text parameters. Not supported yet: md5 authentication, the binary
//! format, notifications, and `COPY`.
//!
//! Every message the server sends is length-checked before it is parsed, so a
//! truncated or hostile server produces an error instead of a panic.

use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use super::pg_tls::TlsStream;
use rustls::RootCertStore;

use crate::pg_message::{
    cstr, frame, i32_at, parse_data_row, parse_row_description, printable, server_error,
};
use crate::pg_scram::Scram;
use crate::pg_url::{parse_url, Ssl, Target};
pub(crate) use crate::pg_message::Outcome;

const PROTOCOL: i32 = 196_608; // 3.0
const SSL_REQUEST: i32 = 80_877_103;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_secs(30);
/// Larger messages are almost certainly a broken peer, not a real result.
const MAX_MESSAGE: usize = 256 << 20;

pub(crate) struct Conn {
    reader: Option<BufReader<TlsStream>>,
    target: Target,
}

impl Conn {
    fn reader_mut(&mut self) -> &mut BufReader<TlsStream> {
        self.reader.as_mut().unwrap()
    }

    pub(crate) fn open(url: &str) -> Result<Conn, String> {
        let target = parse_url(url)?;
        let addr = crate::pg_url::resolve(&target.host, target.port)?;
        let stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|e| format!("cannot connect to {}:{}: {e}", target.host, target.port))?;
        let _ = stream.set_nodelay(true);
        let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
        let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
        let tls = TlsStream::from_plain_tcp(stream);
        let mut conn = Conn {
            reader: Some(BufReader::new(tls)),
            target,
        };
        conn.login()?;
        Ok(conn)
    }

    /// Ask for TLS when the URL allows it, then start up and authenticate.
    fn login(&mut self) -> Result<(), String> {
        if self.target.ssl == Ssl::Prefer {
            self.write_raw(&[
                0,
                0,
                0,
                8,
                (SSL_REQUEST >> 24) as u8,
                (SSL_REQUEST >> 16) as u8,
                (SSL_REQUEST >> 8) as u8,
                SSL_REQUEST as u8,
            ])?;
            let mut reply = [0u8; 1];
            self.reader_mut()
                .read_exact(&mut reply)
                .map_err(|e| format!("the server did not answer the TLS request: {e}"))?;
            match reply[0] {
                b'N' => {}
                b'S' => {
                    let roots = Arc::new(RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.to_vec()));
                    let mut tls = self.reader.take().unwrap().into_inner();
                    tls = tls.upgrade(&self.target.host, roots)
                        .map_err(|e| format!("TLS handshake failed: {e}"))?;
                    self.reader = Some(BufReader::new(tls));
                }
                other => return Err(format!("unexpected reply to the TLS request: {other:#x}")),
            }
        }

        let mut body = Vec::new();
        body.extend_from_slice(&PROTOCOL.to_be_bytes());
        for (key, value) in [
            ("user", self.target.user.as_str()),
            ("database", self.target.database.as_str()),
            ("application_name", self.target.application_name.as_str()),
            ("client_encoding", "UTF8"),
        ] {
            body.extend_from_slice(key.as_bytes());
            body.push(0);
            body.extend_from_slice(value.as_bytes());
            body.push(0);
        }
        body.push(0);
        let mut msg = Vec::with_capacity(body.len() + 4);
        msg.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        msg.extend_from_slice(&body);
        self.write_raw(&msg)?;

        let password = self.target.password.clone();
        let mut scram: Option<Scram> = None;
        loop {
            let (kind, body) = self.read_message()?;
            match kind {
                b'R' => {
                    let code = i32_at(&body, 0)?;
                    match code {
                        0 => {}
                        3 => self.send_password(password.as_bytes())?,
                        5 => {
                            return Err(
                                "the server asked for an md5 password. Kode supports scram-sha-256 and \
                                 cleartext only: set the role's password with password_encryption = \
                                 'scram-sha-256' (the default since PostgreSQL 14)"
                                    .into(),
                            )
                        }
                        10 => {
                            let mechanisms = body
                                .get(4..)
                                .ok_or_else(|| "the server's SASL message is truncated".to_string())?
                                .split(|b| *b == 0)
                                .filter(|m| !m.is_empty())
                                .map(|m| String::from_utf8_lossy(m).to_string())
                                .collect::<Vec<_>>();
                            if !mechanisms.iter().any(|m| m == "SCRAM-SHA-256") {
                                return Err(format!(
                                    "the server offers only {}; Kode implements SCRAM-SHA-256",
                                    mechanisms.join(", ")
                                ));
                            }
                            let state = Scram::start(password.as_bytes());
                            let mut msg = b"SCRAM-SHA-256\0".to_vec();
                            msg.extend_from_slice(&(state.client_first.len() as i32).to_be_bytes());
                            msg.extend_from_slice(state.client_first.as_bytes());
                            self.write_message(b'p', &msg)?;
                            scram = Some(state);
                        }
                        11 => {
                            let state = scram.as_mut().ok_or_else(|| {
                                "the server sent SASLContinue before SASL".to_string()
                            })?;
                            let text = std::str::from_utf8(
                                body.get(4..)
                                    .ok_or_else(|| "the server's SASL message is truncated".to_string())?,
                            )
                            .map_err(|_| "the server's SASL challenge is not UTF-8".to_string())?;
                            let reply = state.continue_with(text)?;
                            self.write_message(b'p', reply.as_bytes())?;
                        }
                        12 => {
                            let state = scram.as_ref().ok_or_else(|| {
                                "the server sent SASLFinal before SASL".to_string()
                            })?;
                            let text = std::str::from_utf8(
                                body.get(4..)
                                    .ok_or_else(|| "the server's SASL message is truncated".to_string())?,
                            )
                            .map_err(|_| "the server's SASL response is not UTF-8".to_string())?;
                            state.finish(text)?;
                        }
                        other => {
                            return Err(format!("unsupported authentication method {other}"));
                        }
                    }
                }
                b'S' | b'K' => {}
                b'N' => {}
                b'E' => return Err(server_error(&body)),
                b'Z' => return Ok(()),
                other => {
                    return Err(format!(
                        "unexpected message `{}` during startup",
                        printable(other)
                    ))
                }
            }
        }
    }

    fn send_password(&mut self, password: &[u8]) -> Result<(), String> {
        let mut body = Vec::with_capacity(password.len() + 1);
        body.extend_from_slice(password);
        body.push(0);
        self.write_message(b'p', &body)
    }

    /// `pg.exec`: one round trip, simple query protocol. Several statements run
    /// in one implicit transaction; the last result set is returned.
    pub(crate) fn simple(&mut self, sql: &str) -> Result<Outcome, String> {
        let mut body = Vec::with_capacity(sql.len() + 1);
        body.extend_from_slice(sql.as_bytes());
        body.push(0);
        let mut msg = Vec::new();
        frame(b'Q', &body, &mut msg);
        self.write_raw(&msg)?;
        self.collect()
    }

    /// `pg.query`: Parse/Bind/Describe/Execute/Sync with text parameters.
    /// `None` binds SQL NULL.
    pub(crate) fn query(
        &mut self,
        sql: &str,
        params: &[Option<Vec<u8>>],
    ) -> Result<Outcome, String> {
        if params.len() > i16::MAX as usize {
            return Err(format!(
                "{} parameters is more than PostgreSQL accepts",
                params.len()
            ));
        }
        let mut msg = Vec::new();

        let mut parse = Vec::new();
        parse.push(0);
        parse.extend_from_slice(sql.as_bytes());
        parse.push(0);
        parse.extend_from_slice(&(params.len() as i16).to_be_bytes());
        for _ in params {
            parse.extend_from_slice(&0i32.to_be_bytes());
        }
        frame(b'P', &parse, &mut msg);

        let mut bind = Vec::new();
        bind.push(0); // unnamed portal
        bind.push(0); // unnamed statement
        bind.extend_from_slice(&0i16.to_be_bytes()); // every parameter is text
        bind.extend_from_slice(&(params.len() as i16).to_be_bytes());
        for param in params {
            match param {
                Some(text) => {
                    bind.extend_from_slice(&(text.len() as i32).to_be_bytes());
                    bind.extend_from_slice(text);
                }
                None => bind.extend_from_slice(&(-1i32).to_be_bytes()),
            }
        }
        bind.extend_from_slice(&0i16.to_be_bytes()); // every column is text
        frame(b'B', &bind, &mut msg);

        frame(b'D', &[b'P', 0], &mut msg);
        let mut execute = Vec::new();
        execute.push(0);
        execute.extend_from_slice(&0i32.to_be_bytes());
        frame(b'E', &execute, &mut msg);
        frame(b'S', &[], &mut msg);

        self.write_raw(&msg)?;
        self.collect()
    }

    /// Read messages until ReadyForQuery, keeping the last result set.
    fn collect(&mut self) -> Result<Outcome, String> {
        let mut out = Outcome::default();
        let mut error: Option<String> = None;
        loop {
            let (kind, body) = self.read_message()?;
            match kind {
                b'T' => out.columns = parse_row_description(&body)?,
                b'D' => out.rows.push(parse_data_row(&body)?),
                b'C' => out.tag = cstr(&body, 0)?.0.to_string(),
                b'E' => error = Some(server_error(&body)),
                b'N' => {
                    if out.notice.is_none() {
                        out.notice = Some(server_error(&body));
                    }
                }
                b'Z' => {
                    return match error {
                        Some(message) => Err(message),
                        None => Ok(out),
                    }
                }
                // ParseComplete, BindComplete, NoData, ParameterDescription,
                // EmptyQueryResponse, PortalSuspended, ParameterStatus,
                // BackendKeyData, NotificationResponse.
                b'1' | b'2' | b'n' | b't' | b'I' | b's' | b'S' | b'K' | b'A' => {}
                other => {
                    return Err(format!(
                        "unexpected message `{}` in a query result",
                        printable(other)
                    ))
                }
            }
        }
    }

    pub(crate) fn close(&mut self) {
        let _ = self.reader.as_mut().unwrap().get_mut().shutdown();
    }

    fn write_message(&mut self, kind: u8, body: &[u8]) -> Result<(), String> {
        let mut msg = Vec::with_capacity(body.len() + 5);
        frame(kind, body, &mut msg);
        self.write_raw(&msg)
    }

    fn write_raw(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.reader_mut()
            .get_mut()
            .write_all(bytes)
            .map_err(|e| format!("sending to the server failed: {e}"))
    }

    fn read_message(&mut self) -> Result<(u8, Vec<u8>), String> {
        let mut head = [0u8; 5];
        self.reader_mut()
            .read_exact(&mut head)
            .map_err(|e| format!("reading from the server failed: {e}"))?;
        let len = i32::from_be_bytes([head[1], head[2], head[3], head[4]]);
        if len < 4 || len as usize - 4 > MAX_MESSAGE {
            return Err(format!("the server sent a message with length {len}"));
        }
        let mut body = vec![0u8; len as usize - 4];
        self.reader_mut()
            .read_exact(&mut body)
            .map_err(|e| format!("the server sent a short message: {e}"))?;
        Ok((head[0], body))
    }
}
