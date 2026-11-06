// HTTP benchmark server, Node standard library only.
const http = require("http");

const port = Number(process.argv[2] || 8080);

const server = http.createServer((req, res) => {
  if (req.url === "/json") {
    const body = JSON.stringify({ message: "hello", ok: true, n: 1 });
    res.writeHead(200, { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(body) });
    res.end(body);
    return;
  }
  const body = "hello";
  res.writeHead(200, { "Content-Type": "text/plain", "Content-Length": body.length });
  res.end(body);
});

// Keep connections open for the whole measurement, like the other servers.
server.keepAliveTimeout = 65_000;
server.headersTimeout = 66_000;

server.listen(port, () => console.log(`listening on ${port}`));
