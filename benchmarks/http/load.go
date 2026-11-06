// A keep-alive HTTP load generator for the benchmark.
//
// One goroutine per connection, each sending `n/c` requests one after another on
// the same socket, so the server sees steady keep-alive traffic with a fixed
// number of open connections. It prints machine-readable lines:
//
//	requests=<n> connections=<c> seconds=<t> rps=<r> p50_ms=<..> p99_ms=<..> errors=<e>
//
// `hey` was measured against these servers first and reported totals that grew
// with the connection count while its own latencies stayed in microseconds, so
// the client is part of the benchmark rather than a black box.
package main

import (
	"bufio"
	"flag"
	"fmt"
	"net"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

type result struct {
	latencies []float64
	errors    int
}

func main() {
	url := flag.String("url", "http://127.0.0.1:8080/hello", "URL to request")
	conns := flag.Int("c", 10, "number of keep-alive connections")
	total := flag.Int("n", 1000, "total number of requests")
	flag.Parse()
	scheme := "http://"
	rest := strings.TrimPrefix(*url, scheme)
	host, path := rest, "/"
	if i := strings.IndexByte(rest, '/'); i >= 0 {
		host, path = rest[:i], rest[i:]
	}
	perConn := *total / *conns
	if perConn < 1 {
		perConn = 1
	}

	results := make([]result, *conns)
	var wg sync.WaitGroup
	start := time.Now()
	for i := 0; i < *conns; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			run(host, path, perConn, &results[idx])
		}(i)
	}
	wg.Wait()
	elapsed := time.Since(start).Seconds()

	var all []float64
	errs := 0
	for _, r := range results {
		all = append(all, r.latencies...)
		errs += r.errors
	}
	sort.Float64s(all)
	pct := func(p float64) float64 {
		if len(all) == 0 {
			return 0
		}
		i := int(p / 100 * float64(len(all)))
		if i >= len(all) {
			i = len(all) - 1
		}
		return all[i]
	}
	rps := 0.0
	if elapsed > 0 {
		rps = float64(len(all)) / elapsed
	}
	fmt.Printf("requests=%d connections=%d seconds=%.3f rps=%.1f p50_ms=%.3f p99_ms=%.3f errors=%d\n",
		len(all), *conns, elapsed, rps, pct(50), pct(99), errs)
	if len(all) == 0 {
		os.Exit(1)
	}
}

func run(host, path string, n int, out *result) {
	conn, err := net.DialTimeout("tcp", host, 5*time.Second)
	if err != nil {
		out.errors += n
		return
	}
	defer conn.Close()
	_ = conn.(*net.TCPConn).SetNoDelay(true)
	reader := bufio.NewReaderSize(conn, 64*1024)
	head := "GET " + path + " HTTP/1.1\r\nHost: " + host + "\r\nConnection: keep-alive\r\n\r\n"

	out.latencies = make([]float64, 0, n)
	for i := 0; i < n; i++ {
		t0 := time.Now()
		if _, err := conn.Write([]byte(head)); err != nil {
			out.errors++
			return
		}
		_ = conn.SetReadDeadline(time.Now().Add(30 * time.Second))
		status, length, err := readResponse(reader)
		if err != nil || status != 200 {
			out.errors++
			return
		}
		if length > 0 {
			if _, err := reader.Discard(length); err != nil {
				out.errors++
				return
			}
		}
		out.latencies = append(out.latencies, float64(time.Since(t0).Microseconds())/1000.0)
	}
}

// readResponse parses the status line and headers, returning the status and the
// Content-Length of the body that follows.
func readResponse(r *bufio.Reader) (status, length int, err error) {
	line, err := r.ReadString('\n')
	if err != nil {
		return 0, 0, err
	}
	fields := strings.Fields(line)
	if len(fields) < 2 {
		return 0, 0, fmt.Errorf("bad status line %q", line)
	}
	status, err = strconv.Atoi(fields[1])
	if err != nil {
		return 0, 0, err
	}
	for {
		h, err := r.ReadString('\n')
		if err != nil {
			return 0, 0, err
		}
		h = strings.TrimRight(h, "\r\n")
		if h == "" {
			return status, length, nil
		}
		if name, value, ok := strings.Cut(h, ":"); ok && strings.EqualFold(name, "Content-Length") {
			length, _ = strconv.Atoi(strings.TrimSpace(value))
		}
	}
}
