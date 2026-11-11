// idle opens N keep-alive connections, sends one request on each, and then
// holds every socket open without writing anything. It exists to answer one
// question for ADR 0037: what does a live idle connection cost the server, and
// what happens above the connection cap?
//
// It reports how many connections were established, how many the server closed
// while they sat idle (a cap or an idle timeout shedding load), how many were
// still open at the end, and the connect + first-response latency.
//
//	go build -o idle idle.go
//	./idle -url http://server:8080/hello -conns 1000 -hold 5s
package main

import (
	"bufio"
	"flag"
	"fmt"
	"io"
	"net"
	"net/http"
	"sort"
	"strings"
	"sync"
	"time"
)

type outcome struct {
	conn    net.Conn
	latency float64 // ms, connect + request + first response
	status  int
}

func main() {
	url := flag.String("url", "http://127.0.0.1:8080/hello", "URL requested on every connection")
	conns := flag.Int("conns", 100, "number of connections to open")
	hold := flag.Duration("hold", 5*time.Second, "how long to keep the connections idle after the first response")
	timeout := flag.Duration("timeout", 10*time.Second, "dial and first-response timeout")
	flag.Parse()

	host, path := splitURL(*url)

	var (
		mu       sync.Mutex
		opened   []outcome
		dialFail int
		start    = time.Now()
		wait     sync.WaitGroup
	)

	wait.Add(*conns)
	for i := 0; i < *conns; i++ {
		go func() {
			defer wait.Done()
			t0 := time.Now()
			c, err := net.DialTimeout("tcp", host, *timeout)
			if err != nil {
				mu.Lock()
				dialFail++
				mu.Unlock()
				return
			}
			_ = c.SetDeadline(time.Now().Add(*timeout))
			req := fmt.Sprintf("GET %s HTTP/1.1\r\nHost: %s\r\nConnection: keep-alive\r\n\r\n", path, host)
			if _, err := io.WriteString(c, req); err != nil {
				c.Close()
				return
			}
			resp, err := http.ReadResponse(bufio.NewReader(c), nil)
			if err != nil {
				c.Close()
				return
			}
			io.Copy(io.Discard, resp.Body)
			resp.Body.Close()
			_ = c.SetDeadline(time.Time{})

			mu.Lock()
			opened = append(opened, outcome{
				conn:    c,
				latency: float64(time.Since(t0).Microseconds()) / 1000.0,
				status:  resp.StatusCode,
			})
			mu.Unlock()
		}()
	}
	wait.Wait()

	// Hold the sockets open without writing. Anything the server closes in this
	// window is load the server refused to carry.
	time.Sleep(*hold)

	// Probe every connection at once: a sequential probe would take
	// 100ms * N to finish at ten thousand connections.
	var (
		probe       sync.WaitGroup
		closedMu    sync.Mutex
		closedEarly int
	)
	statuses := map[int]int{}
	latency := make([]float64, 0, len(opened))
	probe.Add(len(opened))
	for _, o := range opened {
		go func(o outcome) {
			defer probe.Done()
			defer o.conn.Close()
			_ = o.conn.SetReadDeadline(time.Now().Add(100 * time.Millisecond))
			buf := make([]byte, 1)
			if n, err := o.conn.Read(buf); err != nil && !isTimeout(err) {
				closedMu.Lock()
				closedEarly++
				closedMu.Unlock()
			} else if err == nil {
				_ = n // unexpected data, but the connection is still up
			}
		}(o)
	}
	probe.Wait()
	for _, o := range opened {
		statuses[o.status]++
		latency = append(latency, o.latency)
	}

	sort.Float64s(latency)
	pick := func(p float64) float64 {
		if len(latency) == 0 {
			return 0
		}
		return latency[int(p*float64(len(latency)-1))]
	}

	codes := make([]string, 0, len(statuses))
	for code, n := range statuses {
		codes = append(codes, fmt.Sprintf("%d:%d", code, n))
	}
	sort.Strings(codes)

	fmt.Printf("conns=%d established=%d dial_failed=%d closed_early=%d still_open=%d statuses=%s "+
		"connect_ms_p50=%.2f connect_ms_p99=%.2f hold_ms=%d wall_s=%.1f\n",
		*conns, len(opened), dialFail, closedEarly, len(opened)-closedEarly, strings.Join(codes, ","),
		pick(0.50), pick(0.99), hold.Milliseconds(), time.Since(start).Seconds())
}

// splitURL returns "host:port" and the request path for an http:// URL.
func splitURL(url string) (string, string) {
	rest := url
	if i := strings.Index(rest, "://"); i >= 0 {
		rest = rest[i+3:]
	}
	path := "/"
	if i := strings.IndexByte(rest, '/'); i >= 0 {
		path = rest[i:]
		rest = rest[:i]
	}
	return rest, path
}

func isTimeout(err error) bool {
	if ne, ok := err.(net.Error); ok {
		return ne.Timeout()
	}
	return false
}
