# ADR 0034: TLS in the runtime (client and server)

## Context

Plan item 9.3. Every real service is reached over `https://`, and the HTTP
client refused it with a clear error instead of sending plaintext. The
PostgreSQL client had already picked `rustls` 0.23 plus `webpki-roots` for its
opportunistic SSLRequest upgrade (ADR 0033), so a TLS stack and a root store
were already linked into `kode-rt`; the HTTP side had none of its own.

Constraints that shaped the decision:

- One self-contained binary per program, no shared runtime and no platform TLS
  package (Windows has no OpenSSL by default, and shipping one is out of scope).
- The runtime has no C dependencies today; every crate here costs binary size in
  every compiled program and needs the maintainer's approval.
- The plaintext server is an epoll readiness loop on Linux (one fd per
  connection, non-blocking, one thread per CPU) with a thread-per-connection
  fallback elsewhere.
- `serve(port, handler)` and `serve(port, router)` are compiler special cases:
  the handler is a plain function pointer or a `Router`, not a closure value.

## Decision

1. **rustls, not OpenSSL.** Pure Rust, so the same code links on macOS, Linux and
   Windows, and it reuses the `webpki-roots` store the PostgreSQL client already
   carries. `docs/SPEC` claims of "no TLS library" are removed.
2. **One client transport.** `tls.rs` owns `Transport`, which is either a
   plaintext socket or a `StreamOwned<ClientConnection, TcpStream>`. It builds
   the client configuration once per process, checks the host name, completes the
   handshake eagerly, and names the host in any failure. The PostgreSQL client
   and the HTTP client both use it.
3. **DER certificates for the server, not PEM.** PEM would need a third crate
   (`rustls-pemfile`); DER needs none. `serve_tls` therefore takes file paths to
   DER files, and the startup error tells the user the two `openssl` commands.
4. **TLS server runs on the thread-per-connection path, explicitly.** A rustls
   session is a state machine, so it cannot join the fd-level epoll loop: bytes
   have to pass through `read_tls`/`process_new_packets` per connection. Rather
   than bolt a half-state-machine onto the plaintext hot path, `serve_tls` uses
   its own accept loop: the handshake runs on the connection's thread with a
   5-second timeout before any HTTP byte is parsed, then requests are framed
   into a buffer (the stream cannot be cloned into a reader and a writer).
   Plaintext `serve` keeps the epoll loop unchanged.
5. **No silent fallback.** A missing or unusable certificate is a startup error;
   a failed handshake closes that connection. `--release`-style downgrading does
   not exist.

## Alternatives considered

- **System OpenSSL / platform TLS.** Smaller crate graph, but a system dependency
  on every target and no story on Windows.
- **`rustls-pemfile` for PEM.** Nicer for users (every tutorial emits PEM), one
  more crate in the runtime; rejected for now, revisit if DER-only proves
  hostile.
- **epoll + TLS state machine per connection.** Best throughput, largest change:
  the parser would have to become transport-agnostic and every connection would
  own a session in the slab. Deferred; the plaintext path keeps its scalability.
- **Terminate TLS in a reverse proxy.** Pushes the problem onto users and does
  not help the HTTP client.

## Consequences

- `https://` works end to end: verified against a fixture certificate in tests
  and against `example.com` by hand.
- A TLS server pays one thread per connection, capped at
  `MAX_ACTIVE_CONNECTIONS` (1024), and does not use the epoll loop. Under high
  connection counts its RSS grows with connections; this is recorded as a
  limitation, not a claim.
- `rustls` and `webpki-roots` stay linked into every program (they already were
  after ADR 0033). Hello world is 1,077,600 bytes stripped, over the 1 MB budget;
  making optional runtime parts link-on-use is tracked separately in STATUS.
- Certificates are read once at startup; rotation means restarting the process.
  No SNI-based certificate selection, no client certificates, no HTTP/2.
