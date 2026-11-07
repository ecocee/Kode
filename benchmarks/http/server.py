"""HTTP benchmark server, Python standard library only.

Uses HTTP/1.1 with keep-alive and one thread per connection, which is the
closest the standard library gets to what the other servers do.

Two socket settings are needed for a fair test:
  * disable_nagle_algorithm: http.server sends headers and body as separate
    writes; without TCP_NODELAY, Nagle + delayed ACK add ~40 ms per request.
  * request_queue_size: the default listen backlog is 5, which refuses most
    of 1000 simultaneous connections.
"""

import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

JSON = b'{"message":"hello","ok":true,"n":1}'


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    disable_nagle_algorithm = True

    def do_GET(self):  # noqa: N802 - the name is fixed by BaseHTTPRequestHandler
        if self.path == "/json":
            body, ctype = JSON, "application/json"
        else:
            body, ctype = b"hello", "text/plain"
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):  # keep stdout clean for the harness
        pass


class Server(ThreadingHTTPServer):
    request_queue_size = 4096
    daemon_threads = True


if __name__ == "__main__":
    port = int((sys.argv[1] if len(sys.argv) > 1 else "8080").lstrip(":"))
    server = Server(("0.0.0.0", port), Handler)
    print(f"listening on {port}", flush=True)
    server.serve_forever()