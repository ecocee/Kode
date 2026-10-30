
        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    

        $line = # Kode HTTP Server Example

A real, production-quality HTTP server written entirely in the **Kode** language, powered by Go's `net/http` package under the hood.

## Folder Structure

```
http_server/
├── main.kode          ← entry point — imports routes, starts server
├── routes/
│   ├── hello.kode     ← GET  /api/hello
│   ├── echo.kode      ← POST /api/echo
│   └── info.kode      ← GET  /api/info
├── public/
│   └── index.html     ← frontend UI (static files)
└── README.md
```

## Running

From the workspace root:

```sh
# Run directly from source
kode run examples/http_server/main.kode

# Or compile first, then run the bytecode
kode build examples/http_server/main.kode
kode main.kbc
```

Then open **http://localhost:3000/public/index.html** in your browser.

## How imports work

Each file in `routes/` exports a single function that accepts a server handle and registers its routes:

```kode
// routes/hello.kode
fn helloRoute(server) {
    http.get(server, "/api/hello", fn(req) {
        let name = http.query(req, "name")
        if (name == "") { name = "World" }
        return http.respondJSON(200, "{\"message\": \"Hello, ${name}!\"}")
    })
}
```

`main.kode` is just wiring — 15 lines total:

```kode
import { helloRoute } from "routes/hello"
import { echoRoute }  from "routes/echo"
import { infoRoute }  from "routes/info"

let server = http.newServer(3000)
http.static(server, "/public/", "examples/http_server/public")

helloRoute(server)
echoRoute(server)
infoRoute(server)

http.start(server)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/public/*` | Static file server |
| GET | `/api/hello?name=...` | JSON greeting |
| POST | `/api/echo` | Echoes request body |
| GET | `/api/info` | Server metadata |

## http Module API

```kode
// Server
http.newServer(port)                          // create server handle
http.static(server, "/prefix/", "./dir")     // serve static files
http.start(server)                            // start (blocking, Ctrl+C to stop)

// Routes
http.get(server, "/path",    fn(req) { ... })
http.post(server, "/path",   fn(req) { ... })
http.put(server, "/path",    fn(req) { ... })
http.delete(server, "/path", fn(req) { ... })
http.patch(server, "/path",  fn(req) { ... })

// Response helpers (inside handlers)
http.respond(200, "plain text")
http.respondJSON(200, "{\"key\": \"val\"}")
http.respondHTML(200, "<h1>Hello</h1>")
http.respondWith(200, "text/xml", "<root/>")

// Request helpers (inside handlers)
http.query(req, "param")    // query string value
http.body(req)              // request body as string
http.header(req, "Name")    // request header value
http.method(req)            // "GET", "POST", …
http.path(req)              // "/api/hello"
http.remoteAddr(req)        // "127.0.0.1:PORT"
```

## Production features

- Real `net/http` server — not a mock
- Read/Write/Idle timeouts (15s / 15s / 60s)
- Graceful shutdown on `Ctrl+C` with 10-second drain
- Each request handled via Go's built-in goroutine pool
- Static file serving via `http.FileServer`

.Value
        if ($line -notmatch '^\s*```go') { $line -replace '\bfunc\b', 'fn' } else { $line }
    