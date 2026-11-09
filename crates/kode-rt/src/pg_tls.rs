//! Secure TLS over TCP for the PostgreSQL client.
//!
//! A small wrapper so the wire layer can upgrade a plaintext TCP connection to
//! TLS using rustls without pulling a platform TLS stack into every compiled
//! Kode program. Tradeoffs are measured in `benchmarks/report.md` (binary size
//! and link time).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::ClientConfig;
use rustls::ClientConnection;
use rustls::RootCertStore;
use rustls::StreamOwned;

/// A TCP stream that may be plaintext or TLS. The PostgreSQL wire layer uses
/// this type instead of `TcpStream` so the transport choice is made once during
/// connect / SSLRequest handling, and the rest of the protocol code is the same.
pub(crate) struct TlsStream {
    inner: Option<TcpStream>,
    tls: Mutex<Option<StreamOwned<ClientConnection, TcpStream>>>,
}

impl TlsStream {
    /// Wrap an already-connected plaintext TCP stream (used after taking the
    /// stream back out of a `BufReader` during the SSLRequest exchange).
    pub(crate) fn from_plain_tcp(stream: TcpStream) -> Self {
        TlsStream {
            inner: Some(stream),
            tls: Mutex::new(None),
        }
    }

    /// After the server replies `S` to the SSLRequest, complete the TLS
    /// handshake on the existing TCP stream. This takes ownership of `self`
    /// because the plaintext stream is moved into the rustls session.
    pub(crate) fn upgrade(
        mut self,
        hostname: &str,
        roots: Arc<RootCertStore>,
    ) -> Result<Self, String> {
        let stream = self.inner.take().unwrap();
        let tls = handshake(stream, hostname, &roots)?;
        self.tls = Mutex::new(Some(tls));
        Ok(self)
    }

    pub(crate) fn shutdown(&mut self) -> std::io::Result<()> {
        if let Some(tls) = self.tls.lock().unwrap().take() {
            let (mut conn, mut sock) = tls.into_parts();
            conn.send_close_notify();
            let _ = conn.write_tls(&mut sock);
            let _ = sock.flush();
        } else if let Some(stream) = &mut self.inner {
            stream.shutdown(std::net::Shutdown::Both)?;
        }
        Ok(())
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut tls) = *self.tls.lock().unwrap() {
            return tls.read(buf);
        }
        if let Some(ref mut stream) = self.inner {
            return stream.read(buf);
        }
        Err(std::io::Error::other("TlsStream has no active stream"))
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref mut tls) = *self.tls.lock().unwrap() {
            return tls.write(buf);
        }
        if let Some(ref mut stream) = self.inner {
            return stream.write(buf);
        }
        Err(std::io::Error::other("TlsStream has no active stream"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut tls) = *self.tls.lock().unwrap() {
            return tls.flush();
        }
        if let Some(ref mut stream) = self.inner {
            return stream.flush();
        }
        Err(std::io::Error::other("TlsStream has no active stream"))
    }
}

fn handshake(
    stream: TcpStream,
    hostname: &str,
    roots: &Arc<RootCertStore>,
) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
    let config = ClientConfig::builder()
        .with_root_certificates(Arc::clone(roots))
        .with_no_client_auth();

    let name: ServerName<'static> =
        ServerName::try_from(hostname.to_string()).map_err(|e| {
            format!("hostname {hostname} is not a valid TLS name: {e}")
        })?;

    let client = ClientConnection::new(config.into(), name).map_err(|e| {
        format!("TLS session: {e}")
    })?;

    Ok(StreamOwned::new(client, stream))
}
