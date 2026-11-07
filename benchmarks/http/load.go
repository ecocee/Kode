
package main

import (
	"bufio"
	"flag"
	"fmt"
	"net"
	"net/http"
	"net/textproto"
	"net/url"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

type result struct {
	latencies []float64
	failed    int
	unattempted int
}

func main() {
	rawURL := flag.String("url", "http://127.0.0.1:8080/hello", "URL to request")
	conns := flag.Int("c", 10, "number of keep-alive connections")
	total := flag.Int("n", 1000, "total number of requests")
	timeout := flag.Duration("timeout", 30*time.Second, "request timeout")
	flag.Parse()

	if *conns < 1 || *total < 1 {
		fmt.Fprintln(os.Stderr, "connections and requests must be greater than zero")
		os.Exit(2)
	}

	u, err := url.Parse(*rawURL)
	if err != nil || u.Scheme != "http" || u.Host == "" {
		fmt.Fprintln(os.Stderr, "a valid http:// URL is required")
		os.Exit(2)
	}

	path := u.RequestURI()
	if path == "" {
		path = "/"
	}

	host := u.Host
	results := make([]result, *conns)

	var wg sync.WaitGroup
	start := time.Now()

	for i := 0; i < *conns; i++ {
		// Distribute the remainder instead of silently dropping requests.
		count := *total / *conns
		if i < *total % *conns {
			count++
		}
		if count == 0 {
			continue
		}

		wg.Add(1)
		go func(idx, count int) {
			defer wg.Done()
			run(host, path, count, *timeout, &results[idx])
		}(i, count)
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds()

	var latencies []float64
	failed := 0
	unattempted := 0

	for _, r := range results {
		latencies = append(latencies, r.latencies...)
		failed += r.failed
		unattempted += r.unattempted
	}

	sort.Float64s(latencies)

	percentile := func(p float64) float64 {
		if len(latencies) == 0 {
			return 0
		}
		index := int(p / 100 * float64(len(latencies)))
		if index >= len(latencies) {
			index = len(latencies) - 1
		}
		return latencies[index]
	}

	rps := 0.0
	if elapsed > 0 {
		rps = float64(len(latencies)) / elapsed
	}

	fmt.Printf(
		"requests=%d target_requests=%d connections=%d seconds=%.3f rps=%.1f p50_ms=%.3f p95_ms=%.3f p99_ms=%.3f failed_requests=%d unattempted_requests=%d errors=%d\n",
		len(latencies),
		*total,
		*conns,
		elapsed,
		rps,
		percentile(50),
		percentile(95),
		percentile(99),
		failed,
		unattempted,
		failed,
	)

	if len(latencies) != *total || failed != 0 || unattempted != 0 {
		os.Exit(1)
	}
}

func run(host, path string, n int, timeout time.Duration, out *result) {
	conn, err := net.DialTimeout("tcp", host, 5*time.Second)
	if err != nil {
		out.failed++
		out.unattempted += n - 1
		return
	}
	defer conn.Close()

	tcp, ok := conn.(*net.TCPConn)
	if ok {
		_ = tcp.SetNoDelay(true)
	}

	reader := bufio.NewReaderSize(conn, 64*1024)
	request := "GET " + path + " HTTP/1.1\r\n" +
		"Host: " + host + "\r\n" +
		"Connection: keep-alive\r\n\r\n"

	out.latencies = make([]float64, 0, n)

	for i := 0; i < n; i++ {
		start := time.Now()

		_ = conn.SetDeadline(time.Now().Add(timeout))

		if _, err := conn.Write([]byte(request)); err != nil {
			out.failed++
			out.unattempted += n - i - 1
			return
		}

		status, err := readResponse(reader)
		if err != nil || status != http.StatusOK {
			out.failed++
			out.unattempted += n - i - 1
			return
		}

		out.latencies = append(
			out.latencies,
			float64(time.Since(start).Microseconds())/1000,
		)
	}
}

// readResponse consumes the complete response body, including chunked bodies.
// This is important for servers such as Go net/http that may use chunked
// transfer encoding when Content-Length is not known in advance.
func readResponse(r *bufio.Reader) (int, error) {
	line, err := r.ReadString('\n')
	if err != nil {
		return 0, err
	}

	fields := strings.Fields(line)
	if len(fields) < 2 {
		return 0, fmt.Errorf("invalid HTTP status line: %q", line)
	}

	status, err := strconv.Atoi(fields[1])
	if err != nil {
		return 0, err
	}

	headers, err := textproto.NewReader(r).ReadMIMEHeader()
	if err != nil {
		return 0, err
	}

	if status == http.StatusNoContent ||
		status == http.StatusNotModified ||
		status / 100 == 1 {
		return status, nil
	}

	transferEncoding := strings.ToLower(headers.Get("Transfer-Encoding"))
	if strings.Contains(transferEncoding, "chunked") {
		if err := readChunkedBody(r); err != nil {
			return status, err
		}
		return status, nil
	}

	if value := headers.Get("Content-Length"); value != "" {
		length, err := strconv.ParseInt(value, 10, 64)
		if err != nil || length < 0 {
			return status, fmt.Errorf("invalid Content-Length")
		}
		if _, err := r.Discard(int(length)); err != nil {
			return status, err
		}
		return status, nil
	}

	return status, fmt.Errorf("response has no supported body framing")
}

func readChunkedBody(r *bufio.Reader) error {
	for {
		line, err := r.ReadString('\n')
		if err != nil {
			return err
		}

		sizeText := strings.TrimSpace(strings.SplitN(line, ";", 2)[0])
		size, err := strconv.ParseInt(sizeText, 16, 64)
		if err != nil || size < 0 {
			return fmt.Errorf("invalid chunk size")
		}

		if size == 0 {
			// Consume optional trailer headers.
			_, err := textproto.NewReader(r).ReadMIMEHeader()
			return err
		}

		if _, err := r.Discard(int(size)); err != nil {
			return err
		}

		// Every chunk is followed by CRLF.
		cr, err := r.ReadByte()
		if err != nil {
			return err
		}
		lf, err := r.ReadByte()
		if err != nil {
			return err
		}
		if cr != '\r' || lf != '\n' {
			return fmt.Errorf("invalid chunk terminator")
		}
	}
}
