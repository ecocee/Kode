package bytecode

// server_module.go — Kode native server stdlib module for the Kode VM.
//
// Exposed as "server" namespace in Kode scripts:
//
//   server.newServer(port)           → serverHandle (map)
//   server.get(srv, path, fn)        → nil  (register GET route, fn(req) → map)
//   server.post(srv, path, fn)       → nil  (register POST route, fn(req) → map)
//   server.static(srv, urlPrefix, dir) → nil
//   server.start(srv)                → nil  (blocking; starts the server)
//   server.respond(status, body)     → map  (response helper from inside a handler)
//   server.respondJSON(status, body) → map
//   server.header(req, name)         → string
//   server.query(req, name)          → string
//   server.body(req)                 → string
//   server.method(req)               → string
//   server.path(req)                 → string
//
// A request object passed to handler callbacks is a map[string]interface{}
// with keys: method, path, body, headers (map), query (map).
//
// A response object returned from handler callbacks must have:
//   status  int    (HTTP status code, default 200)
//   body    string
//   headers map[string]string  (optional)
//
// Example Kode usage:
//   let srv = server.newServer(3000)
//   server.get(srv, "/", fn(req) {
//     return server.respond(200, "Hello, World!")
//   })
//   server.start(srv)

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
)

// ─── internal types ──────────────────────────────────────────────────────────

// serverHandle is the Go-side state for a Kode server.
type serverHandle struct {
	port   int
	mux    *http.ServeMux
	server *http.Server
}

// serverRegistry maps a short integer key (kept as a Kode int) to the
// actual Go handle.  This avoids exposing Go pointers into the Kode value
// space and keeps garbage-collection safe.
var serverRegistry = map[int]*serverHandle{}
var serverNextID = 1

// ─── dispatch ────────────────────────────────────────────────────────────────

// callServerModule dispatches server.XXX calls from the VM.
// name is the part after the dot, e.g. "newServer", "get", "start", …
func (vm *VM) callServerModule(fn string, args []interface{}) interface{} {
	switch fn {

	// ── server.newServer(port) ──────────────────────────────────────────────
	case "newServer":
		port := 8080
		if len(args) >= 1 {
			port = vm.toInt(args[0])
		}
		mux := http.NewServeMux()
		h := &serverHandle{
			port: port,
			mux:  mux,
		}
		id := serverNextID
		serverNextID++
		serverRegistry[id] = h

		// Return a Kode map so scripts can store it naturally in a variable.
		return map[string]interface{}{
			"_type": "Server",
			"_id":   id,
			"port":  port,
		}

	// ── server.get(srv, path, fn) ────────────────────────────────────────
	case "get":
		return vm.serverRegisterRoute("GET", args)

	// ── server.post(srv, path, fn) ───────────────────────────────────────
	case "post":
		return vm.serverRegisterRoute("POST", args)

	// ── server.put(srv, path, fn) ────────────────────────────────────────
	case "put":
		return vm.serverRegisterRoute("PUT", args)

	// ── server.delete(srv, path, fn) ─────────────────────────────────────
	case "delete":
		return vm.serverRegisterRoute("DELETE", args)

	// ── server.patch(srv, path, fn) ──────────────────────────────────────
	case "patch":
		return vm.serverRegisterRoute("PATCH", args)

	// ── server.static(srv, urlPrefix, dir) ──────────────────────────────
	case "static":
		if len(args) < 3 {
			return nil
		}
		h := vm.serverFromArg(args[0])
		if h == nil {
			return nil
		}
		prefix := vm.valueToString(args[1])
		dir := vm.valueToString(args[2])
		// Ensure prefix ends with / for StripPrefix to work correctly
		if !strings.HasSuffix(prefix, "/") {
			prefix += "/"
		}
		fs := http.FileServer(http.Dir(dir))
		h.mux.Handle(prefix, http.StripPrefix(prefix, fs))
		return nil

	// ── server.start(srv) ────────────────────────────────────────────────
	case "start":
		if len(args) < 1 {
			return nil
		}
		h := vm.serverFromArg(args[0])
		if h == nil {
			fmt.Fprintln(os.Stderr, "[server] start: invalid server handle")
			return nil
		}
		addr := fmt.Sprintf(":%d", h.port)
		h.server = &http.Server{
			Addr:         addr,
			Handler:      h.mux,
			ReadTimeout:  15 * time.Second,
			WriteTimeout: 15 * time.Second,
			IdleTimeout:  60 * time.Second,
		}

		// Graceful shutdown on SIGINT / SIGTERM
		quit := make(chan os.Signal, 1)
		signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

		go func() {
			<-quit
			ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
			defer cancel()
			if err := h.server.Shutdown(ctx); err != nil {
				fmt.Fprintln(os.Stderr, "[server] shutdown error:", err)
			}
		}()

		if err := h.server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			fmt.Fprintf(os.Stdout, "error: server on port %d failed: %v\n", h.port, err)
			return false
		}
		return true

	// ── server.respond(status, body) ────────────────────────────────────────
	case "respond":
		status := 200
		body := ""
		if len(args) >= 1 {
			status = vm.toInt(args[0])
		}
		if len(args) >= 2 {
			body = vm.valueToString(args[1])
		}
		return map[string]interface{}{
			"status":  status,
			"body":    body,
			"headers": map[string]interface{}{"Content-Type": "text/plain; charset=utf-8"},
		}

	// ── server.respondJSON(status, body) ────────────────────────────────────
	case "respondJSON":
		status := 200
		body := ""
		if len(args) >= 1 {
			status = vm.toInt(args[0])
		}
		if len(args) >= 2 {
			body = vm.valueToString(args[1])
		}
		return map[string]interface{}{
			"status":  status,
			"body":    body,
			"headers": map[string]interface{}{"Content-Type": "application/json; charset=utf-8"},
		}

	// ── server.respondHTML(status, body) ────────────────────────────────────
	case "respondHTML":
		status := 200
		body := ""
		if len(args) >= 1 {
			status = vm.toInt(args[0])
		}
		if len(args) >= 2 {
			body = vm.valueToString(args[1])
		}
		return map[string]interface{}{
			"status":  status,
			"body":    body,
			"headers": map[string]interface{}{"Content-Type": "text/html; charset=utf-8"},
		}

	// ── server.respondWith(status, contentType, body) ───────────────────────
	case "respondWith":
		status := 200
		ct := "text/plain; charset=utf-8"
		body := ""
		if len(args) >= 1 {
			status = vm.toInt(args[0])
		}
		if len(args) >= 2 {
			ct = vm.valueToString(args[1])
		}
		if len(args) >= 3 {
			body = vm.valueToString(args[2])
		}
		return map[string]interface{}{
			"status":  status,
			"body":    body,
			"headers": map[string]interface{}{"Content-Type": ct},
		}

	// ── req accessor helpers ──────────────────────────────────────────────
	case "header":
		// server.header(req, name) → string
		if len(args) >= 2 {
			if req, ok := args[0].(map[string]interface{}); ok {
				name := vm.valueToString(args[1])
				if hdrs, ok2 := req["headers"].(map[string]interface{}); ok2 {
					if v, ok3 := hdrs[http.CanonicalHeaderKey(name)]; ok3 {
						return vm.valueToString(v)
					}
				}
			}
		}
		return ""

	case "query":
		// server.query(req, name) → string
		if len(args) >= 2 {
			if req, ok := args[0].(map[string]interface{}); ok {
				name := vm.valueToString(args[1])
				if q, ok2 := req["query"].(map[string]interface{}); ok2 {
					if v, ok3 := q[name]; ok3 {
						return vm.valueToString(v)
					}
				}
			}
		}
		return ""

	case "body":
		// server.body(req) → string
		if len(args) >= 1 {
			if req, ok := args[0].(map[string]interface{}); ok {
				if b, ok2 := req["body"]; ok2 {
					return vm.valueToString(b)
				}
			}
		}
		return ""

	case "method":
		// server.method(req) → string
		if len(args) >= 1 {
			if req, ok := args[0].(map[string]interface{}); ok {
				if m, ok2 := req["method"]; ok2 {
					return vm.valueToString(m)
				}
			}
		}
		return ""

	case "path":
		// server.path(req) → string
		if len(args) >= 1 {
			if req, ok := args[0].(map[string]interface{}); ok {
				if p, ok2 := req["path"]; ok2 {
					return vm.valueToString(p)
				}
			}
		}
		return ""

	case "remoteAddr":
		// server.remoteAddr(req) → string
		if len(args) >= 1 {
			if req, ok := args[0].(map[string]interface{}); ok {
				if a, ok2 := req["remoteAddr"]; ok2 {
					return vm.valueToString(a)
				}
			}
		}
		return ""

	// ── server.setHeader(resp, name, value) ──────────────────────────────────
	// Returns a new response map with the named header set.
	case "setHeader":
		if len(args) < 3 {
			return args[0]
		}
		resp, ok := args[0].(map[string]interface{})
		if !ok {
			return args[0]
		}
		name := vm.valueToString(args[1])
		value := vm.valueToString(args[2])
		newResp := make(map[string]interface{})
		for k, v := range resp {
			newResp[k] = v
		}
		hdrs := make(map[string]interface{})
		if existing, ok2 := resp["headers"].(map[string]interface{}); ok2 {
			for k, v := range existing {
				hdrs[k] = v
			}
		}
		hdrs[name] = value
		newResp["headers"] = hdrs
		return newResp

	// ── server.cors(resp) ─────────────────────────────────────────────────────
	// Returns a new response map with standard CORS headers added.
	case "cors":
		var resp map[string]interface{}
		if len(args) >= 1 {
			if r, ok := args[0].(map[string]interface{}); ok {
				resp = r
			}
		}
		if resp == nil {
			resp = map[string]interface{}{"status": 200, "body": ""}
		}
		newResp := make(map[string]interface{})
		for k, v := range resp {
			newResp[k] = v
		}
		hdrs := make(map[string]interface{})
		if existing, ok2 := resp["headers"].(map[string]interface{}); ok2 {
			for k, v := range existing {
				hdrs[k] = v
			}
		}
		hdrs["Access-Control-Allow-Origin"] = "*"
		hdrs["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, PATCH, OPTIONS"
		hdrs["Access-Control-Allow-Headers"] = "Content-Type, Authorization, X-Requested-With, Accept"
		hdrs["Access-Control-Max-Age"] = "86400"
		newResp["headers"] = hdrs
		return newResp

	// ── server.getPort(srv) ───────────────────────────────────────────────────
	case "getPort":
		if len(args) >= 1 {
			h := vm.serverFromArg(args[0])
			if h != nil {
				return h.port
			}
		}
		return 0

	// ── server.any(srv, path, handler) ───────────────────────────────────────
	// Register a route that matches any HTTP method.
	case "any":
		return vm.serverRegisterRoute("ANY", args)

	// ── server.timestamp() ───────────────────────────────────────────────────
	// Returns the current Unix timestamp as an integer.
	case "timestamp":
		return int(time.Now().Unix())

	// ── server.log(level, message) ────────────────────────────────────────────
	// Print a timestamped log line to stderr.
	case "log":
		level := "INFO"
		msg := ""
		if len(args) >= 1 {
			level = strings.ToUpper(vm.valueToString(args[0]))
		}
		if len(args) >= 2 {
			msg = vm.valueToString(args[1])
		}
		now := time.Now().Format("2006-01-02 15:04:05")
		fmt.Fprintf(os.Stderr, "[%s] [%s] %s\n", now, level, msg)
		return nil

	// ── server.redirect(status, url) ─────────────────────────────────────────
	// Build a redirect response (301 / 302 / 307 / 308).
	case "redirect":
		status := 302
		url := ""
		if len(args) >= 1 {
			status = vm.toInt(args[0])
		}
		if len(args) >= 2 {
			url = vm.valueToString(args[1])
		}
		return map[string]interface{}{
			"status": status,
			"body":   "",
			"headers": map[string]interface{}{
				"Location":     url,
				"Content-Type": "text/plain; charset=utf-8",
			},
		}
	}

	return nil
}

// ─── helpers ─────────────────────────────────────────────────────────────────

// serverFromArg resolves a Kode server-handle map to a Go *serverHandle.
func (vm *VM) serverFromArg(arg interface{}) *serverHandle {
	if m, ok := arg.(map[string]interface{}); ok {
		if id, ok2 := m["_id"].(int); ok2 {
			return serverRegistry[id]
		}
	}
	return nil
}

// serverRegisterRoute registers a GET/POST/… route on the server.
// args: [serverHandle, path, handlerFn]
func (vm *VM) serverRegisterRoute(method string, args []interface{}) interface{} {
	if len(args) < 3 {
		return nil
	}
	h := vm.serverFromArg(args[0])
	if h == nil {
		fmt.Fprintf(os.Stderr, "[server] register route: invalid server handle for %s\n", method)
		return nil
	}
	pattern := vm.valueToString(args[1])
	handlerFn := args[2] // ClosureValue or string (function name)

	h.mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
		if r.Method != method && method != "ANY" {
			http.Error(w, "Method Not Allowed", http.StatusMethodNotAllowed)
			return
		}

		// Build Kode request map
		bodyBytes, _ := io.ReadAll(r.Body)
		defer r.Body.Close()

		headers := make(map[string]interface{})
		for k, vs := range r.Header {
			headers[k] = strings.Join(vs, ", ")
		}

		query := make(map[string]interface{})
		for k, vs := range r.URL.Query() {
			query[k] = strings.Join(vs, ", ")
		}

		req := map[string]interface{}{
			"method":     r.Method,
			"path":       r.URL.Path,
			"body":       string(bodyBytes),
			"headers":    headers,
			"query":      query,
			"remoteAddr": r.RemoteAddr,
		}

		// Call the Kode handler function synchronously
		resp := vm.applyFn(handlerFn, []interface{}{req})

		// Write the HTTP response from the Kode response map
		vm.serverWriteResponse(w, resp)
	})

	return nil
}

// serverWriteResponse writes a Kode response map to an http.ResponseWriter.
// If resp is a plain string, it writes it as text/plain with status 200.
func (vm *VM) serverWriteResponse(w http.ResponseWriter, resp interface{}) {
	switch r := resp.(type) {
	case map[string]interface{}:
		// Apply user-defined extra headers first
		if hdrs, ok := r["headers"].(map[string]interface{}); ok {
			for k, v := range hdrs {
				w.Header().Set(k, vm.valueToString(v))
			}
		}
		// Determine status code
		status := http.StatusOK
		if s, ok := r["status"]; ok {
			status = vm.toInt(s)
		}
		w.WriteHeader(status)
		// Write body
		if body, ok := r["body"]; ok {
			fmt.Fprint(w, vm.valueToString(body))
		}
	case string:
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		fmt.Fprint(w, r)
	default:
		// Nil or unknown — 204 No Content
		w.WriteHeader(http.StatusNoContent)
	}
}
