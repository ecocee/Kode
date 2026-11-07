// HTTP benchmark server, Node standard library only.
// THREADS > 1 starts that many cluster workers (Node's way of using more cores).
const http = require("http");
const cluster = require("cluster");

const port = Number((process.argv[2] || "8080").replace(/^:/, ""));
const workers = Math.max(1, Number(process.env.THREADS || 1));

if (cluster.isPrimary && workers > 1) {
  for (let i = 0; i < workers; i++) cluster.fork();
  cluster.on("exit", (w, code) => {
    console.error(`worker ${w.process.pid} exited (${code})`);
    process.exit(1);
  });
  console.log(`listening on ${port} workers=${workers}`);
} else {
  const JSON_BODY = JSON.stringify({ message: "hello", ok: true, n: 1 });
  const JSON_LEN = Buffer.byteLength(JSON_BODY);

  const server = http.createServer((req, res) => {
    if (req.url === "/json") {
      res.writeHead(200, { "Content-Type": "application/json", "Content-Length": JSON_LEN });
      res.end(JSON_BODY);
      return;
    }
    res.writeHead(200, { "Content-Type": "text/plain", "Content-Length": 5 });
    res.end("hello");
  });

  // Keep connections open for the whole measurement, like the other servers.
  server.keepAliveTimeout = 65_000;
  server.headersTimeout = 66_000;

  server.listen(port, () => {
    if (workers === 1) console.log(`listening on ${port}`);
  });
}