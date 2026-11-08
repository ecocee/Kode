// PostgreSQL benchmark client for Rust (rust-postgres + r2d2).
//
// One workload per process, one JSON record per case on stdout. Statements are
// prepared once per case and executed with the prepared handle, which is what a
// Rust service does. Concurrency uses one pooled connection per worker, held for
// the whole case like a connection-pooled web handler would.

use std::env;
use std::fs;
use std::sync::Mutex;
use std::time::Instant;

use postgres::types::ToSql;
use postgres::{Client, GenericClient, NoTls, Row};
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{json, Map, Value};

const LANGUAGE: &str = "rust";

// ------------------------------------------------------------------ arguments
struct Args {
    workload: String,
    cases: String,
    rows: i64,
    ops: i64,
    conninfo: String,
    only: Vec<String>,
}

fn parse_args() -> Args {
    let mut args = Args {
        workload: String::new(),
        cases: "/bench/workloads/cases.json".to_string(),
        rows: 10_000,
        ops: 200,
        conninfo: String::new(),
        only: Vec::new(),
    };
    let mut host = env::var("PGHOST").unwrap_or_else(|_| "postgres".into());
    let mut port = env::var("PGPORT").unwrap_or_else(|_| "5432".into());
    let mut user = env::var("PGUSER").unwrap_or_else(|_| "bench".into());
    let mut password = env::var("PGPASSWORD").unwrap_or_else(|_| "bench".into());
    let mut dbname = env::var("PGDATABASE").unwrap_or_else(|_| "kode_bench".into());

    let raw: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i + 1 < raw.len() {
        let key = raw[i].trim_start_matches("--").to_string();
        let value = raw[i + 1].clone();
        match key.as_str() {
            "workload" => args.workload = value,
            "cases" => args.cases = value,
            "rows" => args.rows = value.parse().unwrap_or(args.rows),
            "ops" => args.ops = value.parse().unwrap_or(args.ops),
            "host" => host = value,
            "port" => port = value,
            "user" => user = value,
            "password" => password = value,
            "db" => dbname = value,
            "only" => args.only = value.split(',').map(|s| s.to_string()).collect(),
            _ => {}
        }
        i += 2;
    }
    args.conninfo = format!(
        "host={host} port={port} user={user} password={password} dbname={dbname} \
         sslmode=disable application_name=kode-bench-{LANGUAGE}"
    );
    args
}

// ------------------------------------------------------------------ utilities
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = (sorted.len() - 1) as f64 * p;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

fn round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    (value * factor).round() / factor
}

fn summarise(latencies: &mut Vec<f64>, wall_ms: f64) -> Map<String, Value> {
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ops = latencies.len();
    let throughput = if wall_ms > 0.0 { ops as f64 / (wall_ms / 1000.0) } else { 0.0 };
    let mut out = Map::new();
    out.insert("ops".into(), json!(ops));
    out.insert("wall_ms".into(), json!(round(wall_ms, 3)));
    out.insert("throughput".into(), json!(round(throughput, 2)));
    out.insert("p50_ms".into(), json!(round(percentile(latencies, 0.50), 3)));
    out.insert("p95_ms".into(), json!(round(percentile(latencies, 0.95), 3)));
    out.insert("p99_ms".into(), json!(round(percentile(latencies, 0.99), 3)));
    if let (Some(first), Some(last)) = (latencies.first(), latencies.last()) {
        out.insert("min_ms".into(), json!(round(*first, 3)));
        out.insert("max_ms".into(), json!(round(*last, 3)));
    }
    out
}

fn cpu_ms() -> f64 {
    unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut usage);
        let user = usage.ru_utime.tv_sec as f64 * 1000.0 + usage.ru_utime.tv_usec as f64 / 1000.0;
        let system = usage.ru_stime.tv_sec as f64 * 1000.0 + usage.ru_stime.tv_usec as f64 / 1000.0;
        user + system
    }
}

fn peak_rss_kb() -> u64 {
    if let Ok(text) = fs::read_to_string("/proc/self/status") {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("VmHWM:") {
                if let Some(value) = rest.split_whitespace().next() {
                    return value.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

// ------------------------------------------------------------ parameter formulas
fn gen(name: &str, i: i64, rows: i64, worker: i64) -> Value {
    let statuses = ["new", "paid", "shipped", "cancelled"];
    let categories = ["books", "games", "tools", "music"];
    let cities = ["Bengaluru", "Berlin", "Lisbon", "Osaka"];
    match name {
        "id" => json!(i + 1),
        "k" => json!((i * 37) % rows),
        "k_hi" => json!(((i * 37) % rows) + 25),
        "pick_id" => json!(((i * 7) % rows) + 1),
        "pick_product" => json!(((i * 7) % 1000) + 1),
        "offset" => json!((i * 100) % 1000),
        "qty" => json!((i % 5) + 1),
        "order_id" => json!(((i * 997) % 100000) + 1),
        "app_product_id" | "app_customer_id" => json!(100000 + i),
        "app_order_id" => json!(1000000 + i),
        "worker_id" => json!(worker * 1000000 + i + 1),
        "payload" => json!(format!("payload-{i}")),
        "payload2" => json!(format!("updated-{i}")),
        "tag" => json!(format!("t{}", i % 10)),
        "status" => json!(statuses[(i % 4) as usize]),
        "price" => json!("19.99"),
        "sku" => json!(format!("SKU-NEW-{i}")),
        "name" => json!(format!("name-{i}")),
        "email" => json!(format!("user{i}@example.com")),
        "city" => json!(cities[(i % 4) as usize]),
        "category" => json!(categories[(i % 4) as usize]),
        "name_like" => json!("name-%"),
        other => panic!("unknown parameter generator: {other}"),
    }
}

fn boxed(value: Value) -> Box<dyn ToSql + Sync> {
    if let Some(n) = value.as_i64() {
        Box::new(n)
    } else {
        Box::new(value.as_str().unwrap_or("").to_string())
    }
}

fn params_for(names: &[String], i: i64, rows: i64) -> Vec<Box<dyn ToSql + Sync>> {
    names.iter().map(|n| boxed(gen(n, i, rows, 0))).collect()
}

fn refs(params: &[Box<dyn ToSql + Sync>]) -> Vec<&(dyn ToSql + Sync)> {
    params.iter().map(|p| p.as_ref()).collect()
}

fn chunk_values(names: &[String], start: i64, count: i64, rows: i64) -> Vec<Box<dyn ToSql + Sync>> {
    let mut out = Vec::with_capacity(names.len() * count as usize);
    for i in start..start + count {
        out.extend(params_for(names, i, rows));
    }
    out
}

fn sql_error(error: &postgres::Error) -> (bool, String) {
    let code = error.code().map(|c| c.code().to_string()).unwrap_or_default();
    let text = error.to_string();
    let first = text.lines().next().unwrap_or("").chars().take(200).collect();
    (code == "40P01", first)
}

fn consume(rows: &[Row]) -> i64 {
    let mut count = 0;
    for row in rows {
        for (i, column) in row.columns().iter().enumerate() {
            match column.type_().oid() {
                20 => {
                    let _ = row.get::<_, Option<i64>>(i);
                }
                21 | 23 => {
                    let _ = row.get::<_, Option<i32>>(i);
                }
                16 => {
                    let _ = row.get::<_, Option<bool>>(i);
                }
                _ => {
                    let _ = row.try_get::<_, Option<String>>(i);
                }
            }
        }
        count += 1;
    }
    count
}

// ------------------------------------------------------------------ case shapes
fn stmt_loop(client: &mut Client, case: &Value, rows: i64, iterations: i64) -> Map<String, Value> {
    let sql = case["sql"].as_str().unwrap_or("");
    let names: Vec<String> = case["params"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut latencies = Vec::with_capacity(iterations as usize);
    let (mut affected, mut errors, mut deadlocks) = (0i64, 0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();
    match client.prepare(sql) {
        Ok(stmt) => {
            for i in 0..iterations {
                let params = params_for(&names, i, rows);
                let binding = refs(&params);
                let t0 = Instant::now();
                match client.execute(&stmt, &binding) {
                    Ok(n) => affected += n as i64,
                    Err(error) => {
                        errors += 1;
                        let (dead, text) = sql_error(&error);
                        if dead {
                            deadlocks += 1;
                        }
                        if sample.is_empty() {
                            sample = text;
                        }
                    }
                }
                latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        Err(error) => {
            errors += 1;
            sample = sql_error(&error).1;
        }
    }
    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("deadlocks".into(), json!(deadlocks));
    out.insert("error_sample".into(), json!(sample));
    out
}

fn query_loop(client: &mut Client, case: &Value, rows: i64, iterations: i64) -> Map<String, Value> {
    let sql = case["sql"].as_str().unwrap_or("");
    let names: Vec<String> = case["params"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut latencies = Vec::with_capacity(iterations as usize);
    let (mut fetched, mut errors) = (0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();
    match client.prepare(sql) {
        Ok(stmt) => {
            for i in 0..iterations {
                let params = params_for(&names, i, rows);
                let binding = refs(&params);
                let t0 = Instant::now();
                match client.query(&stmt, &binding) {
                    Ok(rows_out) => fetched += consume(&rows_out),
                    Err(error) => {
                        errors += 1;
                        if sample.is_empty() {
                            sample = sql_error(&error).1;
                        }
                    }
                }
                latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        Err(error) => {
            errors += 1;
            sample = sql_error(&error).1;
        }
    }
    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_fetched".into(), json!(fetched));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out
}

#[allow(clippy::too_many_arguments)]
fn chunk_insert_into<C: GenericClient>(
    client: &mut C,
    prefix: &str,
    names: &[String],
    rows: i64,
    iterations: i64,
    chunk: i64,
    affected: &mut i64,
    errors: &mut i64,
    sample: &mut String,
    latencies: &mut Vec<f64>,
) {
    let width = names.len() as i64;
    let mut s = 0;
    while s < iterations {
        let n = chunk.min(iterations - s);
        let mut tuples = Vec::new();
        for r in 0..n {
            let marks: Vec<String> = (0..width)
                .map(|j| format!("${}", r * width + j + 1))
                .collect();
            tuples.push(format!("({})", marks.join(",")));
        }
        let sql = format!("{prefix}{}", tuples.join(","));
        let params = chunk_values(names, s, n, rows);
        let binding = refs(&params);
        let t0 = Instant::now();
        match client.execute(sql.as_str(), &binding) {
            Ok(count) => *affected += count as i64,
            Err(error) => {
                *errors += 1;
                if sample.is_empty() {
                    *sample = sql_error(&error).1;
                }
            }
        }
        latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
        s += n;
    }
}

fn chunk_insert(
    client: &mut Client,
    case: &Value,
    rows: i64,
    iterations: i64,
    chunk: i64,
    in_txn: bool,
) -> Map<String, Value> {
    let prefix = case["sql"].as_str().unwrap_or("");
    let names: Vec<String> = case["params"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut latencies = Vec::new();
    let (mut affected, mut errors) = (0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();

    if in_txn {
        match client.transaction() {
            Ok(mut tx) => {
                chunk_insert_into(
                    &mut tx, prefix, &names, rows, iterations, chunk,
                    &mut affected, &mut errors, &mut sample, &mut latencies,
                );
                if errors == 0 {
                    if let Err(error) = tx.commit() {
                        errors += 1;
                        sample = sql_error(&error).1;
                    }
                }
            }
            Err(error) => {
                errors += 1;
                sample = sql_error(&error).1;
            }
        }
    } else {
        chunk_insert_into(
            client, prefix, &names, rows, iterations, chunk,
            &mut affected, &mut errors, &mut sample, &mut latencies,
        );
    }

    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out.insert("batches".into(), json!((iterations + chunk - 1) / chunk));
    out
}

fn batch_update(client: &mut Client, case: &Value, rows: i64, iterations: i64, chunk: i64) -> Map<String, Value> {
    let prefix = case["sql"].as_str().unwrap_or("");
    let names: Vec<String> = case["params"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut latencies = Vec::new();
    let (mut affected, mut errors) = (0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();
    let mut s = 0;
    while s < iterations {
        let n = chunk.min(iterations - s);
        let mut tuples = Vec::new();
        for r in 0..n {
            tuples.push(format!("(${}::bigint,${}::text)", r * 2 + 1, r * 2 + 2));
        }
        let sql = format!("{prefix}{}) AS v(id, payload) WHERE bench_writes.id = v.id", tuples.join(","));
        let params = chunk_values(&names, s, n, rows);
        let binding = refs(&params);
        let t0 = Instant::now();
        match client.execute(sql.as_str(), &binding) {
            Ok(count) => affected += count as i64,
            Err(error) => {
                errors += 1;
                if sample.is_empty() {
                    sample = sql_error(&error).1;
                }
            }
        }
        latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
        s += n;
    }
    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out.insert("batches".into(), json!((iterations + chunk - 1) / chunk));
    out
}

fn batch_delete(client: &mut Client, case: &Value, _rows: i64, iterations: i64, chunk: i64) -> Map<String, Value> {
    let sql = case["sql"].as_str().unwrap_or("");
    let mut latencies = Vec::new();
    let (mut affected, mut errors) = (0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();
    let mut s = 0;
    while s < iterations {
        let n = chunk.min(iterations - s);
        let t0 = Instant::now();
        let params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(s + 1), Box::new(s + n)];
        let binding = refs(&params);
        match client.execute(sql, &binding) {
            Ok(count) => affected += count as i64,
            Err(error) => {
                errors += 1;
                if sample.is_empty() {
                    sample = sql_error(&error).1;
                }
            }
        }
        latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
        s += n;
    }
    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out.insert("batches".into(), json!((iterations + chunk - 1) / chunk));
    out
}

fn txn_loop(client: &mut Client, case: &Value, rows: i64, iterations: i64) -> Map<String, Value> {
    let sql = case["sql"].as_str().unwrap_or("");
    let names: Vec<String> = case["params"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut latencies = Vec::with_capacity(iterations as usize);
    let (mut affected, mut errors) = (0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();
    match client.transaction() {
        Ok(mut tx) => match tx.prepare(sql) {
            Ok(stmt) => {
                for i in 0..iterations {
                    let params = params_for(&names, i, rows);
                    let binding = refs(&params);
                    let t0 = Instant::now();
                    match tx.execute(&stmt, &binding) {
                        Ok(n) => affected += n as i64,
                        Err(error) => {
                            errors += 1;
                            if sample.is_empty() {
                                sample = sql_error(&error).1;
                            }
                        }
                    }
                    latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                if errors == 0 {
                    if let Err(error) = tx.commit() {
                        errors += 1;
                        sample = sql_error(&error).1;
                    }
                }
            }
            Err(error) => {
                errors += 1;
                sample = sql_error(&error).1;
            }
        },
        Err(error) => {
            errors += 1;
            sample = sql_error(&error).1;
        }
    }
    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out.insert("transactions".into(), json!(1));
    out
}

const INSERT_ROW: &str = "INSERT INTO bench_writes (id, k, payload, tag) VALUES ($1,$2,$3,$4)";

fn row_params(i: i64, rows: i64) -> Vec<Box<dyn ToSql + Sync>> {
    vec![
        Box::new(i + 1),
        Box::new((i * 37) % rows),
        Box::new(format!("payload-{i}")),
        Box::new(format!("t{}", i % 10)),
    ]
}

enum TxnOutcome {
    Committed,
    RolledBack,
    Recovered,
    ExpectedError,
    Failed(String),
}

/// One transaction, its own scope: a `Transaction` that has to be rolled back is
/// consumed, so the next iteration starts from a clean connection.
///
/// The constraint/recovery scenario needs to inspect a failed transaction and
/// then keep using the same connection, so it drives BEGIN/COMMIT/ROLLBACK with
/// batch_execute instead of a `Transaction` handle (which cannot be reused after
/// an error). Same statements, same isolation, one round trip each.
fn txn_exec(client: &mut Client, scenario: &str, i: i64, per_txn: i64, rows: i64) -> TxnOutcome {
    if scenario == "constraint" || scenario == "recovery" {
        if let Err(error) = client.batch_execute("BEGIN") {
            return TxnOutcome::Failed(sql_error(&error).1);
        }
        let _ = client.execute(INSERT_ROW, &refs(&row_params(i, rows)));
        let duplicate = client.execute(INSERT_ROW, &refs(&row_params(i, rows))).is_err();
        if !duplicate {
            return match client.batch_execute("COMMIT") {
                Ok(()) => TxnOutcome::Committed,
                Err(error) => TxnOutcome::Failed(sql_error(&error).1),
            };
        }
        if let Err(error) = client.batch_execute("ROLLBACK") {
            return TxnOutcome::Failed(sql_error(&error).1);
        }
        return if scenario == "recovery" {
            recover(client, i, rows)
        } else {
            TxnOutcome::ExpectedError
        };
    }

    match client.transaction() {
        Ok(mut tx) => match scenario {
            "commit" => match tx.execute(INSERT_ROW, &refs(&row_params(i, rows))) {
                Ok(_) => commit(tx),
                Err(error) => {
                    let _ = tx.rollback();
                    TxnOutcome::Failed(sql_error(&error).1)
                }
            },
            "rollback" => match tx.execute(INSERT_ROW, &refs(&row_params(i, rows))) {
                Ok(_) => {
                    let _ = tx.rollback();
                    TxnOutcome::RolledBack
                }
                Err(error) => {
                    let _ = tx.rollback();
                    TxnOutcome::Failed(sql_error(&error).1)
                }
            },
            "multi" => {
                for j in 0..per_txn {
                    if let Err(error) = tx.execute(INSERT_ROW, &refs(&row_params(i * per_txn + j, rows))) {
                        let _ = tx.rollback();
                        return TxnOutcome::Failed(sql_error(&error).1);
                    }
                }
                commit(tx)
            }
            other => panic!("unknown transaction scenario: {other}"),
        },
        Err(error) => TxnOutcome::Failed(sql_error(&error).1),
    }
}

fn commit(tx: postgres::Transaction<'_>) -> TxnOutcome {
    match tx.commit() {
        Ok(()) => TxnOutcome::Committed,
        Err(error) => TxnOutcome::Failed(sql_error(&error).1),
    }
}

/// After a failed transaction, a clean transaction on the same connection must work.
fn recover(client: &mut Client, i: i64, rows: i64) -> TxnOutcome {
    match client.transaction() {
        Ok(mut tx) => {
            if tx.execute(INSERT_ROW, &refs(&row_params(1_000_000 + i, rows))).is_err() {
                let _ = tx.rollback();
                return TxnOutcome::Failed("recovery insert failed".to_string());
            }
            match tx.commit() {
                Ok(()) => TxnOutcome::Recovered,
                Err(error) => TxnOutcome::Failed(sql_error(&error).1),
            }
        }
        Err(error) => TxnOutcome::Failed(sql_error(&error).1),
    }
}

fn txn_case(client: &mut Client, case: &Value, rows: i64, iterations: i64) -> Map<String, Value> {
    let scenario = case["scenario"].as_str().unwrap_or("");
    let per_txn = case["per_txn"].as_i64().unwrap_or(1);
    let mut latencies = Vec::new();
    let (mut txns, mut errors, mut affected) = (0i64, 0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();

    for i in 0..iterations {
        let t0 = Instant::now();
        match txn_exec(client, scenario, i, per_txn, rows) {
            TxnOutcome::Committed => {
                txns += 1;
                affected += 1;
            }
            TxnOutcome::RolledBack => txns += 1,
            TxnOutcome::Recovered => {
                errors += 1; // the failed transaction was expected
                txns += 1;
                affected += 1;
            }
            TxnOutcome::ExpectedError => errors += 1,
            TxnOutcome::Failed(message) => {
                errors += 1;
                if sample.is_empty() {
                    sample = message;
                }
            }
        }
        latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
    }

    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("transactions".into(), json!(txns));
    out.insert("rows_affected".into(), json!(affected));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out
}
fn app_order_txn(client: &mut Client, _case: &Value, _rows: i64, iterations: i64) -> Map<String, Value> {
    let mut latencies = Vec::new();
    let (mut errors, mut committed, mut rolled) = (0i64, 0i64, 0i64);
    let mut sample = String::new();
    let started = Instant::now();

    for i in 0..iterations {
        let product = ((i * 7) % 1000) + 1;
        let t0 = Instant::now();
        match client.transaction() {
            Ok(mut tx) => {
                let updated = tx.execute(
                    "UPDATE products SET stock = stock - $1 WHERE id = $2 AND stock >= $3",
                    &[&1i64, &product, &1i64],
                );
                match updated {
                    Ok(1) => {
                        let inserted = tx.execute(
                            "INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status) \
                             VALUES ($1,$2,$3,$4,$5::text::numeric,$6)",
                            &[&(2_000_000 + i), &product, &product, &1i64, &"19.99", &"new"],
                        );
                        match inserted.and_then(|_| tx.commit()) {
                            Ok(()) => committed += 1,
                            Err(error) => {
                                errors += 1;
                                if sample.is_empty() {
                                    sample = sql_error(&error).1;
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        let _ = tx.rollback();
                        rolled += 1;
                    }
                    Err(error) => {
                        errors += 1;
                        if sample.is_empty() {
                            sample = sql_error(&error).1;
                        }
                        let _ = tx.rollback();
                    }
                }
            }
            Err(error) => {
                errors += 1;
                sample = sql_error(&error).1;
            }
        }
        latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
    }

    let mut out = summarise(&mut latencies, started.elapsed().as_secs_f64() * 1000.0);
    out.insert("transactions".into(), json!(committed + rolled));
    out.insert("rows_affected".into(), json!(committed));
    out.insert("rolled_back".into(), json!(rolled));
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out
}

fn make_pool(conninfo: &str, size: u32) -> r2d2::Pool<PostgresConnectionManager<NoTls>> {
    let manager = PostgresConnectionManager::new(conninfo.parse().unwrap(), NoTls);
    r2d2::Pool::builder()
        .max_size(size)
        .min_idle(Some(size))
        .build(manager)
        .expect("pool")
}

fn conc(conninfo: &str, case: &Value, rows: i64, ops: i64) -> Map<String, Value> {
    let mode = case["mode"].as_str().unwrap_or("read").to_string();
    let workers = case["workers"].as_i64().unwrap_or(10).max(1) as usize;
    let latencies = Mutex::new(Vec::<f64>::new());
    let errors = Mutex::new(0i64);
    let deadlocks = Mutex::new(0i64);
    let sample = Mutex::new(String::new());

    let setup_started = Instant::now();
    let pool = make_pool(conninfo, workers as u32);
    let mut connections = Vec::with_capacity(workers);
    for _ in 0..workers {
        connections.push(pool.get().expect("pooled connection"));
    }
    let setup_ms = setup_started.elapsed().as_secs_f64() * 1000.0;

    let started = Instant::now();
    std::thread::scope(|scope| {
        for (worker, mut conn) in connections.into_iter().enumerate() {
            let latencies = &latencies;
            let errors = &errors;
            let deadlocks = &deadlocks;
            let sample = &sample;
            let mode = mode.clone();
            scope.spawn(move || {
                let read_stmt = conn.prepare("SELECT id, payload FROM bench_events WHERE id = $1").ok();
                let write_stmt = conn.prepare(INSERT_ROW).ok();
                let mut local = Vec::with_capacity(ops as usize);
                for j in 0..ops {
                    let write = mode == "write" || (mode == "mixed" && j % 10 >= 7);
                    let t0 = Instant::now();
                    let result = if write {
                        match &write_stmt {
                            Some(stmt) => conn
                                .execute(
                                    stmt,
                                    &[
                                        &(worker as i64 * 1_000_000 + j + 1),
                                        &((j * 37) % rows),
                                        &format!("payload-w{worker}"),
                                        &format!("t{}", j % 10),
                                    ],
                                )
                                .map(|_| ()),
                            None => Ok(()),
                        }
                    } else {
                        match &read_stmt {
                            Some(stmt) => {
                                let id = (((worker as i64 * ops + j) * 7) % rows) + 1;
                                conn.query(stmt, &[&id]).map(|rows| {
                                    consume(&rows);
                                })
                            }
                            None => Ok(()),
                        }
                    };
                    if let Err(error) = result {
                        *errors.lock().unwrap() += 1;
                        let (dead, text) = sql_error(&error);
                        if dead {
                            *deadlocks.lock().unwrap() += 1;
                        }
                        let mut guard = sample.lock().unwrap();
                        if guard.is_empty() {
                            *guard = text;
                        }
                    }
                    local.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                latencies.lock().unwrap().extend(local);
            });
        }
    });
    let wall = started.elapsed().as_secs_f64() * 1000.0;

    let mut all = latencies.into_inner().unwrap();
    let mut out = summarise(&mut all, wall);
    out.insert("errors".into(), json!(errors.into_inner().unwrap()));
    out.insert("deadlocks".into(), json!(deadlocks.into_inner().unwrap()));
    out.insert("workers".into(), json!(workers));
    out.insert("connections".into(), json!(workers));
    out.insert("setup_ms".into(), json!(round(setup_ms, 3)));
    out.insert("error_sample".into(), json!(sample.into_inner().unwrap()));
    out
}

fn conn_case(conninfo: &str, case: &Value, rows: i64, iterations: i64) -> Map<String, Value> {
    let scenario = case["scenario"].as_str().unwrap_or("");
    let mut latencies = Vec::new();
    let (mut errors, mut setup_ms, mut connections) = (0i64, 0f64, 1i64);
    let mut sample = String::new();

    match scenario {
        "connect" => {
            for _ in 0..iterations {
                let t0 = Instant::now();
                match Client::connect(conninfo, NoTls) {
                    Ok(mut client) => {
                        if let Err(error) = client.simple_query("SELECT 1") {
                            errors += 1;
                            sample = sql_error(&error).1;
                        }
                    }
                    Err(error) => {
                        errors += 1;
                        if sample.is_empty() {
                            sample = sql_error(&error).1;
                        }
                    }
                }
                latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        "reuse" => {
            let t0 = Instant::now();
            let mut client = Client::connect(conninfo, NoTls).expect("connect");
            setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let stmt = client.prepare("SELECT id, payload FROM bench_events WHERE id = $1").ok();
            for i in 0..iterations {
                let t = Instant::now();
                if let Some(stmt) = &stmt {
                    if let Err(error) = client.query(stmt, &[&((((i * 7) % rows) + 1))]) {
                        errors += 1;
                        if sample.is_empty() {
                            sample = sql_error(&error).1;
                        }
                    }
                }
                latencies.push(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        "pool" | "exhaust" => {
            let size = if scenario == "pool" { 10 } else { 4 };
            let t0 = Instant::now();
            let pool = make_pool(conninfo, size);
            let mut warm = Vec::new();
            for _ in 0..size {
                warm.push(pool.get().expect("pooled connection"));
            }
            setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
            connections = size as i64;
            if scenario == "pool" {
                let mut client = warm.pop().expect("connection");
                let stmt = client.prepare("SELECT id, payload FROM bench_events WHERE id = $1").ok();
                for i in 0..iterations {
                    let t = Instant::now();
                    if let Some(stmt) = &stmt {
                        if let Err(error) = client.query(stmt, &[&((((i * 7) % rows) + 1))]) {
                            errors += 1;
                            if sample.is_empty() {
                                sample = sql_error(&error).1;
                            }
                        }
                    }
                    latencies.push(t.elapsed().as_secs_f64() * 1000.0);
                }
            } else {
                drop(warm);
                let inner_lat = Mutex::new(Vec::<f64>::new());
                let errors_m = Mutex::new(0i64);
                let sample_m = Mutex::new(String::new());
                std::thread::scope(|scope| {
                    for w in 0..20usize {
                        let pool = &pool;
                        let inner_lat = &inner_lat;
                        let errors_m = &errors_m;
                        let sample_m = &sample_m;
                        scope.spawn(move || {
                            let mut local = Vec::new();
                            for j in 0..iterations {
                                let t = Instant::now();
                                let id = ((w as i64 * iterations + j) % rows) + 1;
                                match pool.get() {
                                    Ok(mut conn) => {
                                        if let Err(error) = conn.query("SELECT id FROM bench_events WHERE id = $1", &[&id]) {
                                            *errors_m.lock().unwrap() += 1;
                                            let mut guard = sample_m.lock().unwrap();
                                            if guard.is_empty() {
                                                *guard = sql_error(&error).1;
                                            }
                                        }
                                    }
                                    Err(_) => *errors_m.lock().unwrap() += 1,
                                }
                                local.push(t.elapsed().as_secs_f64() * 1000.0);
                            }
                            inner_lat.lock().unwrap().extend(local);
                        });
                    }
                });
                latencies.extend(inner_lat.into_inner().unwrap());
                errors += errors_m.into_inner().unwrap();
                if sample.is_empty() {
                    sample = sample_m.into_inner().unwrap();
                }
            }
        }
        "recover" => {
            let mut control = Client::connect(conninfo, NoTls).expect("connect");
            for _ in 0..iterations {
                let t0 = Instant::now();
                if let Ok(mut victim) = Client::connect(conninfo, NoTls) {
                    if let Ok(row) = victim.query_one("SELECT pg_backend_pid()", &[]) {
                        let pid: i32 = row.get(0);
                        let _ = control.execute("SELECT pg_terminate_backend($1)", &[&pid]);
                        if victim.query("SELECT 1", &[]).is_err() {
                            errors += 1; // expected: the backend was terminated
                        }
                    }
                }
                if control.query("SELECT count(*) FROM bench_events WHERE id = 1", &[]).is_err() {
                    errors += 1;
                }
                latencies.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            connections = 2;
        }
        other => panic!("unknown connection scenario: {other}"),
    }

    let wall: f64 = latencies.iter().sum();
    let mut out = summarise(&mut latencies, wall);
    out.insert("errors".into(), json!(errors));
    out.insert("error_sample".into(), json!(sample));
    out.insert("setup_ms".into(), json!(round(setup_ms, 3)));
    out.insert("connections".into(), json!(connections));
    out.insert("reconnect_ok".into(), json!(scenario == "recover" && errors == iterations));
    out
}

fn run_checks(client: &mut Client, case: &Value, rows: i64) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    if let Some(checks) = case["checks"].as_array() {
        for check in checks {
            let sql = check["sql"].as_str().unwrap_or("");
            let names: Vec<String> = check["params"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let params = params_for(&names, 0, rows);
            let binding = refs(&params);
            match client.query(sql, &binding) {
                Ok(rows_out) => {
                    let mut row_values = Vec::new();
                    if let Some(row) = rows_out.first() {
                        for i in 0..row.columns().len() {
                            let text = match row.columns()[i].type_().oid() {
                                20 => row.get::<_, Option<i64>>(i).map(|v| v.to_string()),
                                21 | 23 => row.get::<_, Option<i32>>(i).map(|v| v.to_string()),
                                16 => row.get::<_, Option<bool>>(i).map(|v| v.to_string()),
                                _ => row.try_get::<_, Option<String>>(i).ok().flatten(),
                            };
                            row_values.push(text.unwrap_or_else(|| "NULL".to_string()));
                        }
                    }
                    out.push(row_values);
                }
                Err(error) => out.push(vec![format!("ERROR: {}", sql_error(&error).1)]),
            }
        }
    }
    out
}

fn main() {
    let args = parse_args();
    let text = fs::read_to_string(&args.cases).expect("cases.json");
    let spec: Value = serde_json::from_str(&text).expect("cases.json");
    let chunk = spec["chunk"].as_i64().unwrap_or(500);

    // File order is significant: several workloads rely on the last case leaving
    // the dataset in a state the workload invariants assert.
    let cases: Vec<&Value> = spec["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .filter(|c| c["workload"].as_str() == Some(args.workload.as_str()))
        .filter(|c| {
            args.only.is_empty()
                || args
                    .only
                    .iter()
                    .any(|id| c["id"].as_str() == Some(id.as_str()))
        })
        .collect();

    if cases.is_empty() {
        println!("{}", json!({"language": LANGUAGE, "workload": args.workload, "error": "no cases"}));
        std::process::exit(2);
    }

    let mut client = Client::connect(&args.conninfo, NoTls).expect("connect");

    for case in cases {
        let mut iterations = match case["iterations"].as_i64() {
            Some(n) => n,
            None => args.rows,
        };
        if let Some(setup) = case["setup"].as_str() {
            let sql = setup.replace("{rows}", &args.rows.to_string());
            if let Err(error) = client.simple_query(&sql) {
                eprintln!("setup failed for {}: {error}", case["id"].as_str().unwrap_or(""));
                std::process::exit(1);
            }
        }
        let started_cpu = cpu_ms();
        let t0 = Instant::now();
        let shape = case["shape"].as_str().unwrap_or("");
        let mut result = match shape {
            "stmt_loop" => stmt_loop(&mut client, case, args.rows, iterations),
            "query_loop" => query_loop(&mut client, case, args.rows, iterations),
            "chunk_insert" => chunk_insert(&mut client, case, args.rows, iterations, chunk, false),
            "chunk_insert_txn" => chunk_insert(&mut client, case, args.rows, iterations, chunk, true),
            "batch_update" => batch_update(&mut client, case, args.rows, iterations, chunk),
            "batch_delete" => batch_delete(&mut client, case, args.rows, iterations, chunk),
            "txn_loop" => txn_loop(&mut client, case, args.rows, iterations),
            "txn_case" => txn_case(&mut client, case, args.rows, iterations),
            "app_order_txn" => app_order_txn(&mut client, case, args.rows, iterations),
            "conc" => {
                let workers = case["workers"].as_i64().unwrap_or(10);
                iterations = args.ops * workers;
                conc(&args.conninfo, case, args.rows, args.ops)
            }
            "conn" => conn_case(&args.conninfo, case, args.rows, iterations),
            other => panic!("unknown shape: {other}"),
        };
        let case_cpu = cpu_ms() - started_cpu;
        let checks = run_checks(&mut client, case, args.rows);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;

        let mut record = Map::new();
        record.insert("language".into(), json!(LANGUAGE));
        record.insert("workload".into(), json!(args.workload));
        record.insert("case".into(), json!(case["id"].as_str().unwrap_or("")));
        record.insert("shape".into(), json!(shape));
        record.insert("rows".into(), json!(args.rows));
        record.insert("iterations".into(), json!(iterations));
        record.insert("expect_errors".into(), json!(case["expect_errors"].as_bool().unwrap_or(false)));
        record.insert("checks".into(), json!(checks));
        record.insert("cpu_ms".into(), json!(round(case_cpu, 3)));
        record.insert("peak_rss_kb".into(), json!(peak_rss_kb()));
        record.insert("elapsed_ms".into(), json!(round(elapsed, 3)));
        record.insert("status".into(), json!("ok"));
        if !result.contains_key("setup_ms") {
            record.insert("setup_ms".into(), json!(0.0));
        }
        if !result.contains_key("error_sample") {
            record.insert("error_sample".into(), json!(""));
        }
        if !result.contains_key("throughput") {
            record.insert("throughput".into(), json!(0.0));
        }
        for (key, value) in result.iter() {
            record.insert(key.clone(), value.clone());
        }
        let _ = &mut result;
        println!("{}", Value::Object(record));
    }
}
