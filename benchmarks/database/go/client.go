// PostgreSQL benchmark client for Go (database/sql + lib/pq).
//
// One workload per process, one JSON record per case on stdout. Prepared
// statements are prepared once per case and reused, which is what a Go service
// does; concurrency uses one held *sql.Conn per worker so each worker really
// runs on its own connection.
package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/lib/pq"
)

const language = "go"

var (
	statuses   = []string{"new", "paid", "shipped", "cancelled"}
	categories = []string{"books", "games", "tools", "music"}
	cities     = []string{"Bengaluru", "Berlin", "Lisbon", "Osaka"}
)

type generator struct {
	workload string
	id       int
	text     string
}

type caseSpec struct {
	ID           string `json:"id"`
	Workload     string `json:"workload"`
	Shape        string `json:"shape"`
	Scenario     string `json:"scenario"`
	Mode         string `json:"mode"`
	SQL          string `json:"sql"`
	Setup        string `json:"setup"`
	Iterations   any    `json:"iterations"`
	PerTxn       int    `json:"per_txn"`
	Workers      int    `json:"workers"`
	ExpectErrors bool   `json:"expect_errors"`
	Params       []string
	Checks       []struct {
		SQL    string   `json:"sql"`
		Params []string `json:"params"`
	} `json:"checks"`
}

type spec struct {
	Chunk int        `json:"chunk"`
	Cases []caseSpec `json:"cases"`
}

func nowMs() float64 { return float64(time.Now().UnixNano()) / 1e6 }

func round(v float64, digits int) float64 {
	f := 1.0
	for i := 0; i < digits; i++ {
		f *= 10
	}
	return float64(int64(v*f+0.5)) / f
}

func percentile(sorted []float64, p float64) float64 {
	if len(sorted) == 0 {
		return 0
	}
	pos := float64(len(sorted)-1) * p
	lo := int(pos)
	hi := lo + 1
	if hi >= len(sorted) {
		return sorted[len(sorted)-1]
	}
	return sorted[lo] + (sorted[hi]-sorted[lo])*(pos-float64(lo))
}

func summarise(lat []float64, wallMs float64) map[string]any {
	sorted := append([]float64(nil), lat...)
	sort.Float64s(sorted)
	ops := len(sorted)
	tput := 0.0
	if wallMs > 0 {
		tput = float64(ops) / (wallMs / 1000.0)
	}
	out := map[string]any{
		"ops":        ops,
		"wall_ms":    round(wallMs, 3),
		"throughput": round(tput, 2),
		"p50_ms":     round(percentile(sorted, 0.50), 3),
		"p95_ms":     round(percentile(sorted, 0.95), 3),
		"p99_ms":     round(percentile(sorted, 0.99), 3),
	}
	if ops > 0 {
		out["min_ms"] = round(sorted[0], 3)
		out["max_ms"] = round(sorted[ops-1], 3)
	}
	return out
}

func cpuMs() float64 {
	var ru syscall.Rusage
	syscall.Getrusage(syscall.RUSAGE_SELF, &ru)
	return float64(ru.Utime.Sec+ru.Stime.Sec)*1000.0 +
		float64(ru.Utime.Usec+ru.Stime.Usec)/1000.0
}

func peakRssKb() int {
	data, err := os.ReadFile("/proc/self/status")
	if err != nil {
		return 0
	}
	for _, line := range strings.Split(string(data), "\n") {
		if strings.HasPrefix(line, "VmHWM:") {
			fields := strings.Fields(line)
			if len(fields) > 1 {
				v, _ := strconv.Atoi(fields[1])
				return v
			}
		}
	}
	return 0
}

// ------------------------------------------------------------ parameter formulas
func gen(name string, i, rows, worker int) any {
	switch name {
	case "id":
		return i + 1
	case "k":
		return (i * 37) % rows
	case "k_hi":
		return ((i * 37) % rows) + 25
	case "pick_id":
		return ((i * 7) % rows) + 1
	case "pick_product":
		return ((i * 7) % 1000) + 1
	case "offset":
		return (i * 100) % 1000
	case "qty":
		return (i % 5) + 1
	case "order_id":
		return ((i * 997) % 100000) + 1
	case "app_product_id", "app_customer_id":
		return 100000 + i
	case "app_order_id":
		return 1000000 + i
	case "worker_id":
		return worker*1000000 + i + 1
	case "payload":
		return "payload-" + strconv.Itoa(i)
	case "payload2":
		return "updated-" + strconv.Itoa(i)
	case "tag":
		return "t" + strconv.Itoa(i%10)
	case "status":
		return statuses[i%4]
	case "price":
		return "19.99"
	case "sku":
		return "SKU-NEW-" + strconv.Itoa(i)
	case "name":
		return "name-" + strconv.Itoa(i)
	case "email":
		return "user" + strconv.Itoa(i) + "@example.com"
	case "city":
		return cities[i%4]
	case "category":
		return categories[i%4]
	case "name_like":
		return "name-%"
	}
	panic("unknown parameter generator: " + name)
}

func paramsFor(names []string, i, rows int) []any {
	out := make([]any, len(names))
	for k, n := range names {
		out[k] = gen(n, i, rows, 0)
	}
	return out
}

func chunkValues(names []string, start, count, rows int) []any {
	out := make([]any, 0, len(names)*count)
	for i := start; i < start+count; i++ {
		out = append(out, paramsFor(names, i, rows)...)
	}
	return out
}

func isDeadlock(err error) bool {
	var pqErr *pq.Error
	return errors.As(err, &pqErr) && pqErr.Code == "40P01"
}

func isUnique(err error) bool {
	var pqErr *pq.Error
	return errors.As(err, &pqErr) && pqErr.Code == "23505"
}

func firstLine(err error) string {
	text := err.Error()
	if idx := strings.Index(text, "\n"); idx >= 0 {
		text = text[:idx]
	}
	if len(text) > 200 {
		text = text[:200]
	}
	return text
}

// ------------------------------------------------------------------ case shapes
func stmtLoop(db *sql.DB, c caseSpec, rows, iterations int) map[string]any {
	stmt, err := db.Prepare(c.SQL)
	if err != nil {
		return map[string]any{"error_sample": firstLine(err), "errors": 1}
	}
	defer stmt.Close()
	lat := make([]float64, 0, iterations)
	affected, errCount := 0, 0
	sample := ""
	start := nowMs()
	for i := 0; i < iterations; i++ {
		t0 := nowMs()
		res, err := stmt.Exec(paramsFor(c.Params, i, rows)...)
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else if n, e := res.RowsAffected(); e == nil && n > 0 {
			affected += int(n)
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	return out
}

func queryLoop(db *sql.DB, c caseSpec, rows, iterations int) map[string]any {
	stmt, err := db.Prepare(c.SQL)
	if err != nil {
		return map[string]any{"error_sample": firstLine(err), "errors": 1}
	}
	defer stmt.Close()
	lat := make([]float64, 0, iterations)
	fetched, errCount := 0, 0
	sample := ""
	start := nowMs()
	for i := 0; i < iterations; i++ {
		t0 := nowMs()
		rows_, err := stmt.Query(paramsFor(c.Params, i, rows)...)
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else {
			cols, _ := rows_.Columns()
			vals := make([]any, len(cols))
			ptrs := make([]any, len(cols))
			for k := range vals {
				ptrs[k] = &vals[k]
			}
			for rows_.Next() {
				if rows_.Scan(ptrs...) == nil {
					fetched++
				}
			}
			rows_.Close()
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["rows_fetched"] = fetched
	out["errors"] = errCount
	out["error_sample"] = sample
	return out
}

func chunkInsert(db *sql.DB, c caseSpec, rows, iterations, chunk int, inTxn bool) map[string]any {
	width := len(c.Params)
	lat := []float64{}
	affected, errCount := 0, 0
	sample := ""
	start := nowMs()
	var tx *sql.Tx
	var err error
	if inTxn {
		tx, err = db.Begin()
		if err != nil {
			return map[string]any{"error_sample": firstLine(err), "errors": 1}
		}
	}
	for s := 0; s < iterations; s += chunk {
		n := chunk
		if s+n > iterations {
			n = iterations - s
		}
		placeholders := make([]string, n)
		for r := 0; r < n; r++ {
			marks := make([]string, width)
			for j := 0; j < width; j++ {
				marks[j] = "$" + strconv.Itoa(r*width+j+1)
			}
			placeholders[r] = "(" + strings.Join(marks, ",") + ")"
		}
		sqlText := c.SQL + strings.Join(placeholders, ",")
		args := chunkValues(c.Params, s, n, rows)
		t0 := nowMs()
		var res sql.Result
		if inTxn {
			res, err = tx.Exec(sqlText, args...)
		} else {
			res, err = db.Exec(sqlText, args...)
		}
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else if nr, e := res.RowsAffected(); e == nil && nr > 0 {
			affected += int(nr)
		}
		lat = append(lat, nowMs()-t0)
	}
	if inTxn && tx != nil {
		if errCount == 0 {
			if err := tx.Commit(); err != nil {
				errCount++
				sample = firstLine(err)
			}
		} else {
			tx.Rollback()
		}
	}
	out := summarise(lat, nowMs()-start)
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	out["batches"] = (iterations + chunk - 1) / chunk
	return out
}

func batchUpdate(db *sql.DB, c caseSpec, rows, iterations, chunk int) map[string]any {
	lat := []float64{}
	affected, errCount := 0, 0
	sample := ""
	start := nowMs()
	for s := 0; s < iterations; s += chunk {
		n := chunk
		if s+n > iterations {
			n = iterations - s
		}
		parts := make([]string, n)
		for r := 0; r < n; r++ {
			parts[r] = fmt.Sprintf("($%d::bigint,$%d::text)", r*2+1, r*2+2)
		}
		sqlText := c.SQL + strings.Join(parts, ",") + ") AS v(id, payload) WHERE bench_writes.id = v.id"
		t0 := nowMs()
		res, err := db.Exec(sqlText, chunkValues(c.Params, s, n, rows)...)
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else if nr, e := res.RowsAffected(); e == nil && nr > 0 {
			affected += int(nr)
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	out["batches"] = (iterations + chunk - 1) / chunk
	return out
}

func batchDelete(db *sql.DB, c caseSpec, rows, iterations, chunk int) map[string]any {
	stmt, err := db.Prepare(c.SQL)
	if err != nil {
		return map[string]any{"error_sample": firstLine(err), "errors": 1}
	}
	defer stmt.Close()
	lat := []float64{}
	affected, errCount := 0, 0
	sample := ""
	start := nowMs()
	for s := 0; s < iterations; s += chunk {
		n := chunk
		if s+n > iterations {
			n = iterations - s
		}
		t0 := nowMs()
		res, err := stmt.Exec(s+1, s+n)
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else if nr, e := res.RowsAffected(); e == nil && nr > 0 {
			affected += int(nr)
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	out["batches"] = (iterations + chunk - 1) / chunk
	return out
}

func txnLoop(db *sql.DB, c caseSpec, rows, iterations int) map[string]any {
	tx, err := db.Begin()
	if err != nil {
		return map[string]any{"error_sample": firstLine(err), "errors": 1}
	}
	stmt, err := tx.Prepare(c.SQL)
	if err != nil {
		tx.Rollback()
		return map[string]any{"error_sample": firstLine(err), "errors": 1}
	}
	lat := make([]float64, 0, iterations)
	affected, errCount := 0, 0
	sample := ""
	start := nowMs()
	for i := 0; i < iterations; i++ {
		t0 := nowMs()
		res, err := stmt.Exec(paramsFor(c.Params, i, rows)...)
		if err != nil {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
		} else if n, e := res.RowsAffected(); e == nil && n > 0 {
			affected += int(n)
		}
		lat = append(lat, nowMs()-t0)
	}
	stmt.Close()
	if errCount == 0 {
		if err := tx.Commit(); err != nil {
			errCount++
			sample = firstLine(err)
		}
	} else {
		tx.Rollback()
	}
	out := summarise(lat, nowMs()-start)
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	out["transactions"] = 1
	return out
}

const insertRow = "INSERT INTO bench_writes (id, k, payload, tag) VALUES ($1,$2,$3,$4)"

func rowValues(i, rows int) []any {
	return []any{i + 1, (i * 37) % rows, "payload-" + strconv.Itoa(i), "t" + strconv.Itoa(i%10)}
}

func txnCase(db *sql.DB, c caseSpec, rows, iterations int) map[string]any {
	lat := []float64{}
	txns, errCount, affected := 0, 0, 0
	sample := ""
	start := nowMs()
	for i := 0; i < iterations; i++ {
		t0 := nowMs()
		tx, err := db.Begin()
		if err != nil {
			errCount++
			sample = firstLine(err)
			break
		}
		failed := false
		switch c.Scenario {
		case "commit", "rollback":
			_, err = tx.Exec(insertRow, rowValues(i, rows)...)
		case "multi":
			for j := 0; j < c.PerTxn; j++ {
				if _, err = tx.Exec(insertRow, rowValues(i*c.PerTxn+j, rows)...); err != nil {
					break
				}
			}
		case "constraint", "recovery":
			if _, err = tx.Exec(insertRow, rowValues(i, rows)...); err == nil {
				if _, err = tx.Exec(insertRow, rowValues(i, rows)...); err != nil {
					if !isUnique(err) {
						failed = true
					} else {
						errCount++
					}
					tx.Rollback()
					if c.Scenario == "recovery" && !failed {
						if tx2, e2 := db.Begin(); e2 == nil {
							if _, e3 := tx2.Exec(insertRow, rowValues(1000000+i, rows)...); e3 == nil {
								if e4 := tx2.Commit(); e4 == nil {
									txns++
									affected++
								}
							} else {
								tx2.Rollback()
							}
						}
					}
					lat = append(lat, nowMs()-t0)
					continue
				}
			}
		}
		if err != nil && !failed {
			errCount++
			if sample == "" {
				sample = firstLine(err)
			}
			tx.Rollback()
		} else if c.Scenario == "rollback" {
			tx.Rollback()
			txns++
		} else {
			if e := tx.Commit(); e != nil {
				errCount++
				sample = firstLine(e)
			} else {
				txns++
				affected++
			}
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["transactions"] = txns
	out["rows_affected"] = affected
	out["errors"] = errCount
	out["error_sample"] = sample
	return out
}

func appOrderTxn(db *sql.DB, c caseSpec, rows, iterations int) map[string]any {
	lat := []float64{}
	errCount, committed, rolled := 0, 0, 0
	sample := ""
	start := nowMs()
	for i := 0; i < iterations; i++ {
		product := ((i * 7) % 1000) + 1
		t0 := nowMs()
		tx, err := db.Begin()
		if err != nil {
			errCount++
			sample = firstLine(err)
			break
		}
		res, err := tx.Exec("UPDATE products SET stock = stock - $1 WHERE id = $2 AND stock >= $3", 1, product, 1)
		if err != nil {
			errCount++
			sample = firstLine(err)
			tx.Rollback()
			lat = append(lat, nowMs()-t0)
			continue
		}
		n, _ := res.RowsAffected()
		if n == 1 {
			_, err = tx.Exec(`INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status)
				VALUES ($1,$2,$3,$4,$5::text::numeric,$6)`, 2000000+i, product, product, 1, "19.99", "new")
			if err == nil && tx.Commit() == nil {
				committed++
			} else {
				errCount++
				sample = firstLine(err)
				tx.Rollback()
			}
		} else {
			tx.Rollback()
			rolled++
		}
		lat = append(lat, nowMs()-t0)
	}
	out := summarise(lat, nowMs()-start)
	out["transactions"] = committed + rolled
	out["rows_affected"] = committed
	out["rolled_back"] = rolled
	out["errors"] = errCount
	out["error_sample"] = sample
	return out
}

func conc(ctx context.Context, db *sql.DB, c caseSpec, rows, ops int) map[string]any {
	workers := c.Workers
	if workers == 0 {
		workers = 10
	}
	db.SetMaxOpenConns(workers)
	lat := []float64{}
	var mu sync.Mutex
	errCount, deadlocks := 0, 0
	sample := ""

	setupStart := nowMs()
	conns := make([]*sql.Conn, workers)
	for i := 0; i < workers; i++ {
		conn, err := db.Conn(ctx)
		if err != nil {
			return map[string]any{"error_sample": firstLine(err), "errors": 1}
		}
		conns[i] = conn
	}
	setupMs := nowMs() - setupStart

	start := nowMs()
	var wg sync.WaitGroup
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func(worker int) {
			defer wg.Done()
			conn := conns[worker]
			readStmt, _ := conn.PrepareContext(ctx, "SELECT id, payload FROM bench_events WHERE id = $1")
			writeStmt, _ := conn.PrepareContext(ctx, insertRow)
			if readStmt != nil {
				defer readStmt.Close()
			}
			if writeStmt != nil {
				defer writeStmt.Close()
			}
			local := make([]float64, 0, ops)
			localErr, localDead := 0, 0
			localSample := ""
			for j := 0; j < ops; j++ {
				write := c.Mode == "write" || (c.Mode == "mixed" && j%10 >= 7)
				t0 := nowMs()
				var err error
				if write {
					_, err = writeStmt.Exec(worker*1000000+j+1, (j*37)%rows, "payload-w"+strconv.Itoa(worker), "t"+strconv.Itoa(j%10))
				} else {
					var rowsOut *sql.Rows
					rowsOut, err = readStmt.Query((((worker*ops+j)*7)%rows)+1)
					if err == nil {
						rowsOut.Close()
					}
				}
				if err != nil {
					localErr++
					if isDeadlock(err) {
						localDead++
					}
					if localSample == "" {
						localSample = firstLine(err)
					}
				}
				local = append(local, nowMs()-t0)
			}
			mu.Lock()
			lat = append(lat, local...)
			errCount += localErr
			deadlocks += localDead
			if sample == "" {
				sample = localSample
			}
			mu.Unlock()
		}(w)
	}
	wg.Wait()
	wall := nowMs() - start
	for _, conn := range conns {
		conn.Close()
	}
	out := summarise(lat, wall)
	out["errors"] = errCount
	out["deadlocks"] = deadlocks
	out["workers"] = workers
	out["connections"] = workers
	out["setup_ms"] = round(setupMs, 3)
	out["error_sample"] = sample
	return out
}

func openDB(dsn string, max int) *sql.DB {
	db, err := sql.Open("postgres", dsn)
	if err != nil {
		panic(err)
	}
	db.SetMaxOpenConns(max)
	db.SetMaxIdleConns(max)
	return db
}

func connCase(ctx context.Context, dsn string, c caseSpec, rows, iterations int) map[string]any {
	lat := []float64{}
	errCount := 0
	sample := ""
	setupMs := 0.0
	connections := 1

	switch c.Scenario {
	case "connect":
		for i := 0; i < iterations; i++ {
			t0 := nowMs()
			db, err := sql.Open("postgres", dsn)
			if err != nil {
				errCount++
				continue
			}
			db.SetMaxOpenConns(1)
			if err := db.PingContext(ctx); err != nil {
				errCount++
				sample = firstLine(err)
			}
			db.Close()
			lat = append(lat, nowMs()-t0)
		}
	case "reuse":
		t0 := nowMs()
		db := openDB(dsn, 1)
		if err := db.PingContext(ctx); err != nil {
			errCount++
			sample = firstLine(err)
		}
		setupMs = nowMs() - t0
		stmt, _ := db.Prepare("SELECT id, payload FROM bench_events WHERE id = $1")
		for i := 0; i < iterations; i++ {
			t := nowMs()
			r, err := stmt.Query((((i * 7) % rows) + 1))
			if err != nil {
				errCount++
				sample = firstLine(err)
			} else {
				r.Close()
			}
			lat = append(lat, nowMs()-t)
		}
		stmt.Close()
		db.Close()
	case "pool":
		t0 := nowMs()
		db := openDB(dsn, 10)
		for i := 0; i < 10; i++ {
			conn, err := db.Conn(ctx)
			if err == nil {
				conn.Close()
			}
		}
		setupMs = nowMs() - t0
		stmt, _ := db.Prepare("SELECT id, payload FROM bench_events WHERE id = $1")
		for i := 0; i < iterations; i++ {
			t := nowMs()
			r, err := stmt.Query((((i * 7) % rows) + 1))
			if err != nil {
				errCount++
				sample = firstLine(err)
			} else {
				r.Close()
			}
			lat = append(lat, nowMs()-t)
		}
		stmt.Close()
		db.Close()
		connections = 10
	case "exhaust":
		t0 := nowMs()
		db := openDB(dsn, 4)
		for i := 0; i < 4; i++ {
			conn, err := db.Conn(ctx)
			if err == nil {
				conn.Close()
			}
		}
		setupMs = nowMs() - t0
		var mu sync.Mutex
		var wg sync.WaitGroup
		for w := 0; w < 20; w++ {
			wg.Add(1)
			go func(worker int) {
				defer wg.Done()
				stmt, _ := db.Prepare("SELECT id FROM bench_events WHERE id = $1")
				defer stmt.Close()
				for j := 0; j < iterations; j++ {
					t := nowMs()
					r, err := stmt.Query((((worker*iterations + j) % rows) + 1))
					mu.Lock()
					if err != nil {
						errCount++
						if sample == "" {
							sample = firstLine(err)
						}
					} else {
						r.Close()
					}
					lat = append(lat, nowMs()-t)
					mu.Unlock()
				}
			}(w)
		}
		wg.Wait()
		db.Close()
		connections = 4
	case "recover":
		control := openDB(dsn, 1)
		defer control.Close()
		for i := 0; i < iterations; i++ {
			t0 := nowMs()
			victim := openDB(dsn, 1)
			var pid int
			if err := victim.QueryRow("SELECT pg_backend_pid()").Scan(&pid); err != nil {
				errCount++
				sample = firstLine(err)
			} else {
				var killed bool
				control.QueryRow("SELECT pg_terminate_backend($1)", pid).Scan(&killed)
				var one int
				if err := victim.QueryRow("SELECT 1").Scan(&one); err != nil {
					errCount++ // expected: this backend was terminated
				}
				var count int
				if err := control.QueryRow("SELECT count(*) FROM bench_events WHERE id = 1").Scan(&count); err != nil {
					errCount++
				}
			}
			victim.Close()
			lat = append(lat, nowMs()-t0)
		}
		connections = 2
	default:
		panic("unknown connection scenario: " + c.Scenario)
	}

	wall := 0.0
	for _, v := range lat {
		wall += v
	}
	out := summarise(lat, wall)
	out["errors"] = errCount
	out["error_sample"] = sample
	out["setup_ms"] = round(setupMs, 3)
	out["connections"] = connections
	out["reconnect_ok"] = c.Scenario == "recover" && errCount == iterations
	return out
}

func runChecks(db *sql.DB, c caseSpec, rows int) [][]string {
	out := [][]string{}
	for _, check := range c.Checks {
		rowsOut, err := db.Query(check.SQL, paramsFor(check.Params, 0, rows)...)
		if err != nil {
			out = append(out, []string{"ERROR: " + firstLine(err)})
			continue
		}
		cols, _ := rowsOut.Columns()
		vals := make([]any, len(cols))
		ptrs := make([]any, len(cols))
		for k := range vals {
			ptrs[k] = &vals[k]
		}
		if rowsOut.Next() {
			if err := rowsOut.Scan(ptrs...); err == nil {
				row := make([]string, len(vals))
				for k, v := range vals {
					row[k] = render(v)
				}
				out = append(out, row)
			}
		}
		rowsOut.Close()
	}
	return out
}

func render(v any) string {
	if v == nil {
		return "NULL"
	}
	switch t := v.(type) {
	case []byte:
		return string(t)
	case bool:
		if t {
			return "true"
		}
		return "false"
	}
	return fmt.Sprint(v)
}

func main() {
	workload := flag.String("workload", "", "workload to run")
	casesPath := flag.String("cases", "/bench/workloads/cases.json", "cases file")
	rows := flag.Int("rows", 10000, "dataset rows")
	ops := flag.Int("ops", 200, "ops per concurrent worker")
	host := flag.String("host", envOr("PGHOST", "postgres"), "host")
	port := flag.String("port", envOr("PGPORT", "5432"), "port")
	user := flag.String("user", envOr("PGUSER", "bench"), "user")
	password := flag.String("password", envOr("PGPASSWORD", "bench"), "password")
	dbname := flag.String("db", envOr("PGDATABASE", "kode_bench"), "database")
	only := flag.String("only", "", "comma separated case ids")
	flag.Parse()

	data, err := os.ReadFile(*casesPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	var s spec
	if err := json.Unmarshal(data, &s); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	wanted := map[string]bool{}
	if *only != "" {
		for _, id := range strings.Split(*only, ",") {
			wanted[id] = true
		}
	}
	cases := []caseSpec{}
	for _, c := range s.Cases {
		if c.Workload == *workload && (len(wanted) == 0 || wanted[c.ID]) {
			cases = append(cases, c)
		}
	}
	if len(cases) == 0 {
		fmt.Printf("{\"language\":\"%s\",\"workload\":\"%s\",\"error\":\"no cases\"}\n", language, *workload)
		os.Exit(2)
	}

	dsn := fmt.Sprintf("host=%s port=%s user=%s password=%s dbname=%s sslmode=disable application_name=kode-bench-%s",
		*host, *port, *user, *password, *dbname, language)
	db := openDB(dsn, 1)
	defer db.Close()
	ctx := context.Background()

	for _, c := range cases {
		iterations := 0
		switch v := c.Iterations.(type) {
		case string:
			iterations = *rows
		case float64:
			iterations = int(v)
		default:
			iterations = *rows
		}
		if c.Setup != "" {
			if _, err := db.Exec(strings.ReplaceAll(c.Setup, "{rows}", strconv.Itoa(*rows))); err != nil {
				fmt.Fprintf(os.Stderr, "setup failed for %s: %v\n", c.ID, err)
				os.Exit(1)
			}
		}
		startedCpu := cpuMs()
		t0 := nowMs()
		var result map[string]any
		switch c.Shape {
		case "stmt_loop":
			result = stmtLoop(db, c, *rows, iterations)
		case "query_loop":
			result = queryLoop(db, c, *rows, iterations)
		case "chunk_insert":
			result = chunkInsert(db, c, *rows, iterations, s.Chunk, false)
		case "chunk_insert_txn":
			result = chunkInsert(db, c, *rows, iterations, s.Chunk, true)
		case "batch_update":
			result = batchUpdate(db, c, *rows, iterations, s.Chunk)
		case "batch_delete":
			result = batchDelete(db, c, *rows, iterations, s.Chunk)
		case "txn_loop":
			result = txnLoop(db, c, *rows, iterations)
		case "txn_case":
			result = txnCase(db, c, *rows, iterations)
		case "app_order_txn":
			result = appOrderTxn(db, c, *rows, iterations)
		case "conc":
			result = conc(ctx, db, c, *rows, *ops)
			iterations = *ops * c.Workers
		case "conn":
			result = connCase(ctx, dsn, c, *rows, iterations)
		default:
			panic("unknown shape: " + c.Shape)
		}
		caseCpu := cpuMs() - startedCpu
		checks := runChecks(db, c, *rows)

		record := map[string]any{
			"language":      language,
			"workload":      *workload,
			"case":          c.ID,
			"shape":         c.Shape,
			"rows":          *rows,
			"iterations":    iterations,
			"expect_errors": c.ExpectErrors,
			"checks":        checks,
			"cpu_ms":        round(caseCpu, 3),
			"peak_rss_kb":   peakRssKb(),
			"elapsed_ms":    round(nowMs()-t0, 3),
			"status":        "ok",
		}
		if _, ok := result["setup_ms"]; !ok {
			record["setup_ms"] = 0.0
		}
		if _, ok := result["error_sample"]; !ok {
			record["error_sample"] = ""
		}
		if _, ok := result["throughput"]; !ok {
			record["throughput"] = 0.0
		}
		for k, v := range result {
			record[k] = v
		}
		line, _ := json.Marshal(record)
		fmt.Println(string(line))
	}
}

func envOr(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}
