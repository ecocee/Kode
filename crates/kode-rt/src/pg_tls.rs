//! Secure TLS over TCP for the PostgreSQL client.
//!
//! A small wrapper so the wire layer can upgrade a plaintext TCP connection to
//! TLS using rustls without pulling a platform TLS stack into every compiled
//! Kode program. Tradeoffs are measured in `benchmarks/report.md` (binary size
//! and link time).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;

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
    /// Open a plaintext TCP connection. Caller performs the PostgreSQL
    /// SSLRequest exchange and upgrades only if the server offers TLS.
    pub(crate) fn plaintext(host: &str, port: u16) -> Result<Self, String> {
        let addr = (host, port)
            .to_socket_addrs()
            .map_err(|e| format!("cannot resolve {host}:{port}: {e}"))?
            .next()
            .ok_or_else(|| format!("{host}:{port} has no address"))?;

        let inner = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
            .map_err(|e| format!("cannot connect to {host}:{port}: {e}"))?;
        let _ = inner.set_nodelay(true);
        let _ = inner.set_read_timeout(Some(Duration::from_secs(30)));
        let _ = inner.set_write_timeout(Some(Duration::from_secs(30)));

        Ok(TlsStream {
            inner: Some(inner),
            tls: Mutex::new(None),
        })
    }

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
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TlsStream has no active stream",
        ))
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
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TlsStream has no active stream",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut tls) = *self.tls.lock().unwrap() {
            return tls.flush();
        }
        if let Some(ref mut stream) = self.inner {
            return stream.flush();
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TlsStream has no active stream",
        ))
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
