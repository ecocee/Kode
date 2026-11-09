//! TLS over TCP for clients.
//!
//! rustls with the webpki root store, so no platform TLS stack is needed and a
//! program that talks TLS still links on Windows without OpenSSL. The client
//! configuration is built once per process and shared.
//!
//! `Transport` is the single place a connection decides between plaintext and
//! TLS; the PostgreSQL and HTTP clients read and write through it unchanged.
//! Tradeoffs are measured in `benchmarks/report.md` (binary size and link time).

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, OnceLock};

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

/// A connected TCP stream that may speak plaintext or TLS.
pub(crate) enum Transport {
    Plain(TcpStream),
    /// Boxed: a rustls session is much larger than a socket, and this value is
    /// moved into and out of `BufReader` on every TLS upgrade.
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Transport {
    pub(crate) fn plain(stream: TcpStream) -> Self {
        Transport::Plain(stream)
    }

    /// Complete a TLS handshake on this connection, verifying the peer
    /// certificate against the webpki roots and the host name.
    ///
    /// The handshake runs eagerly so an expired certificate or a peer that does
    /// not speak TLS is reported here, with the host name, instead of surfacing
    /// as a confusing I/O error on the first read.
    pub(crate) fn upgrade(self, hostname: &str) -> Result<Self, String> {
        self.upgrade_with(hostname, client_config())
    }

    pub(crate) fn upgrade_with(
        self,
        hostname: &str,
        config: Arc<ClientConfig>,
    ) -> Result<Self, String> {
        let Transport::Plain(stream) = self else {
            return Err("this connection is already using TLS".to_string());
        };

        let name = ServerName::try_from(hostname.to_string()).map_err(|e| {
            format!("`{hostname}` is not a valid TLS host name: {e}")
        })?;
        let session = ClientConnection::new(config, name)
            .map_err(|e| format!("cannot start TLS with {hostname}: {e}"))?;

        let mut tls = StreamOwned::new(session, stream);
        while tls.conn.is_handshaking() {
            tls.conn
                .complete_io(&mut tls.sock)
                .map_err(|e| format!("TLS handshake with {hostname} failed: {e}"))?;
        }
        Ok(Transport::Tls(Box::new(tls)))
    }

    /// Send a TLS close notification, or close the socket when plaintext.
    pub(crate) fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Transport::Plain(stream) => stream.shutdown(Shutdown::Both),
            Transport::Tls(tls) => {
                tls.conn.send_close_notify();
                let _ = tls.conn.write_tls(&mut tls.sock);
                let _ = tls.sock.flush();
                Ok(())
            }
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Transport::Plain(stream) => stream.read(buf),
            Transport::Tls(tls) => tls.read(buf),
        }
    }
}

impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Transport::Plain(stream) => stream.write(buf),
            Transport::Tls(tls) => tls.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Transport::Plain(stream) => stream.flush(),
            Transport::Tls(tls) => tls.flush(),
        }
    }
}

/// The process-wide client configuration: webpki roots, no client certificate.
pub(crate) fn client_config() -> Arc<ClientConfig> {
    static CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(|| Arc::new(build_config(root_store())))
        .clone()
}

/// A configuration with explicit roots, so a test can trust its own certificate.
#[cfg(test)]
pub(crate) fn client_config_with_roots(roots: RootCertStore) -> Arc<ClientConfig> {
    Arc::new(build_config(roots))
}

fn build_config(roots: RootCertStore) -> ClientConfig {
    ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

fn root_store() -> RootCertStore {
    RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// A plaintext connection reads and writes like the socket it wraps.
    #[test]
    fn plaintext_reads_and_writes() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 5];
            sock.read_exact(&mut buf).unwrap();
            sock.write_all(&buf).unwrap();
        });

        let stream = TcpStream::connect(addr).unwrap();
        let mut transport = Transport::plain(stream);
        transport.write_all(b"hello").unwrap();
        transport.flush().unwrap();
        let mut back = [0u8; 5];
        transport.read_exact(&mut back).unwrap();
        assert_eq!(&back, b"hello");
        server.join().unwrap();
    }

    /// A host name that cannot appear in a certificate is rejected before any
    /// bytes are written.
    #[test]
    fn upgrade_rejects_an_invalid_host_name() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();

        match Transport::plain(stream).upgrade("bad name") {
            Ok(_) => panic!("a host name with a space must not be accepted"),
            Err(message) => assert!(
                message.contains("not a valid TLS host name"),
                "unexpected message: {message}"
            ),
        }
    }

    /// A peer that answers in plaintext fails the handshake with a message that
    /// names the host, instead of hanging or panicking.
    #[test]
    fn upgrade_reports_a_peer_that_is_not_tls() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf);
            let _ = sock.write_all(b"HTTP/1.0 400 Bad Request\r\n\r\n");
            let _ = sock.shutdown(Shutdown::Both);
        });

        let stream = TcpStream::connect(addr).unwrap();
        let message = match Transport::plain(stream).upgrade("localhost") {
            Ok(_) => panic!("a plaintext peer must not complete a TLS handshake"),
            Err(message) => message,
        };
        assert!(
            message.contains("TLS handshake with localhost failed"),
            "unexpected message: {message}"
        );
        server.join().unwrap();
    }

    /// The webpki configuration is built once and shared.
    #[test]
    fn client_config_is_cached() {
        assert!(Arc::ptr_eq(&client_config(), &client_config()));
    }
}
