#!/usr/bin/env node
'use strict';
// PostgreSQL benchmark client for Node.js (pg 8).
//
// Node's concurrency is asynchronous, not threaded: the concurrency cases run N
// interleaved async workers, which is how a Node service talks to PostgreSQL.
// Everything else mirrors the other clients: one workload per process, one JSON
// record per case, latencies measured around each awaited operation.

const fs = require('fs');
const { Client, Pool } = require('pg');

const LANGUAGE = 'node';
const STATUSES = ['new', 'paid', 'shipped', 'cancelled'];
const CATEGORIES = ['books', 'games', 'tools', 'music'];
const CITIES = ['Bengaluru', 'Berlin', 'Lisbon', 'Osaka'];

function parseArgs(argv) {
  const out = {
    workload: '',
    cases: '',
    rows: 10000,
    ops: 200,
    host: process.env.PGHOST || 'postgres',
    port: process.env.PGPORT || '5432',
    user: process.env.PGUSER || 'bench',
    password: process.env.PGPASSWORD || 'bench',
    db: process.env.PGDATABASE || 'kode_bench',
    only: '',
  };
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    const value = argv[i + 1];
    if (!key.startsWith('--')) continue;
    const name = key.slice(2);
    if (name === 'workload') out.workload = value;
    else if (name === 'cases') out.cases = value;
    else if (name === 'rows') out.rows = parseInt(value, 10);
    else if (name === 'ops') out.ops = parseInt(value, 10);
    else if (name === 'host') out.host = value;
    else if (name === 'port') out.port = value;
    else if (name === 'user') out.user = value;
    else if (name === 'password') out.password = value;
    else if (name === 'db') out.db = value;
    else if (name === 'only') out.only = value;
    i += 1;
  }
  return out;
}

const nowMs = () => Number(process.hrtime.bigint()) / 1e6;

function percentile(sorted, p) {
  if (sorted.length === 0) return 0;
  const pos = (sorted.length - 1) * p;
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  if (lo === hi) return sorted[lo];
  return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
}

function peakRssKb() {
  try {
    const text = fs.readFileSync('/proc/self/status', 'utf8');
    const match = text.match(/^VmHWM:\s+(\d+)/m);
    return match ? parseInt(match[1], 10) : 0;
  } catch (err) {
    return 0;
  }
}

function cpuMs() {
  const usage = process.cpuUsage();
  return (usage.user + usage.system) / 1000;
}

function summarise(latencies, wallMs) {
  const sorted = latencies.slice().sort((a, b) => a - b);
  const ops = sorted.length;
  return {
    ops,
    wall_ms: round(wallMs, 3),
    throughput: wallMs > 0 ? round(ops / (wallMs / 1000), 2) : 0,
    p50_ms: round(percentile(sorted, 0.5), 3),
    p95_ms: round(percentile(sorted, 0.95), 3),
    p99_ms: round(percentile(sorted, 0.99), 3),
    min_ms: sorted.length ? round(sorted[0], 3) : 0,
    max_ms: sorted.length ? round(sorted[sorted.length - 1], 3) : 0,
  };
}

const round = (value, digits) => {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
};

// ------------------------------------------------------------ parameter formulas
// Duplicated verbatim in every client; documented in workloads/README.md.
function gen(name, i, rows, worker) {
  switch (name) {
    case 'id': return i + 1;
    case 'k': return (i * 37) % rows;
    case 'k_hi': return ((i * 37) % rows) + 25;
    case 'pick_id': return ((i * 7) % rows) + 1;
    case 'pick_product': return ((i * 7) % 1000) + 1;
    case 'offset': return (i * 100) % 1000;
    case 'qty': return (i % 5) + 1;
    case 'order_id': return ((i * 997) % 100000) + 1;
    case 'app_product_id':
    case 'app_customer_id': return 100000 + i;
    case 'app_order_id': return 1000000 + i;
    case 'worker_id': return worker * 1000000 + i + 1;
    case 'payload': return 'payload-' + i;
    case 'payload2': return 'updated-' + i;
    case 'tag': return 't' + (i % 10);
    case 'status': return STATUSES[i % 4];
    case 'price': return '19.99';
    case 'sku': return 'SKU-NEW-' + i;
    case 'name': return 'name-' + i;
    case 'email': return 'user' + i + '@example.com';
    case 'city': return CITIES[i % 4];
    case 'category': return CATEGORIES[i % 4];
    case 'name_like': return 'name-%';
    default: throw new Error('unknown parameter generator: ' + name);
  }
}

const paramsFor = (names, i, rows, worker = 0) => names.map((n) => gen(n, i, rows, worker));

function chunkValues(names, start, count, rows) {
  const flat = [];
  for (let i = start; i < start + count; i += 1) flat.push(...paramsFor(names, i, rows));
  return flat;
}

// Prepared statements are named and cached per SQL text; pg prepares on first
// use and reuses the server-side plan on the same connection.
const statementNames = new Map();
function statementName(sql) {
  if (!statementNames.has(sql)) statementNames.set(sql, 'kode_s' + statementNames.size);
  return statementNames.get(sql);
}

const isDeadlock = (err) => err && err.code === '40P01';
const firstLine = (err) => String(err && err.message ? err.message : err).split('\n')[0].slice(0, 200);

// ------------------------------------------------------------------ case shapes
async function stmtLoop(client, kase, rows, iterations) {
  const names = kase.params || [];
  const lat = [];
  let affected = 0; let errors = 0; let deadlocks = 0; let sample = '';
  const start = nowMs();
  for (let i = 0; i < iterations; i += 1) {
    const t0 = nowMs();
    try {
      const result = await client.query({ name: statementName(kase.sql), text: kase.sql, values: paramsFor(names, i, rows) });
      if (result.rowCount > 0) affected += result.rowCount;
    } catch (err) {
      errors += 1;
      if (isDeadlock(err)) deadlocks += 1;
      sample = sample || firstLine(err);
    }
    lat.push(nowMs() - t0);
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_affected: affected, errors, deadlocks, error_sample: sample };
}

async function queryLoop(client, kase, rows, iterations) {
  const names = kase.params || [];
  const lat = [];
  let fetched = 0; let errors = 0; let sample = '';
  const start = nowMs();
  for (let i = 0; i < iterations; i += 1) {
    const t0 = nowMs();
    try {
      const result = await client.query({ name: statementName(kase.sql), text: kase.sql, values: paramsFor(names, i, rows) });
      fetched += result.rows.length;
    } catch (err) {
      errors += 1;
      sample = sample || firstLine(err);
    }
    lat.push(nowMs() - t0);
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_fetched: fetched, errors, error_sample: sample };
}

async function chunkInsert(client, kase, rows, iterations, chunk, inTxn) {
  const names = kase.params || [];
  const width = names.length;
  const lat = []; let affected = 0; let errors = 0; let sample = '';
  const start = nowMs();
  try {
    if (inTxn) await client.query('BEGIN');
    for (let s = 0; s < iterations; s += chunk) {
      const n = Math.min(chunk, iterations - s);
      const tuples = [];
      for (let r = 0; r < n; r += 1) tuples.push('(' + new Array(width).fill('$').map((_, j) => '$' + (r * width + j + 1)).join(',') + ')');
      const sql = kase.sql + tuples.join(',');
      const t0 = nowMs();
      const result = await client.query({ text: sql, values: chunkValues(names, s, n, rows) });
      if (result.rowCount > 0) affected += result.rowCount;
      lat.push(nowMs() - t0);
    }
    if (inTxn) await client.query('COMMIT');
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
    if (inTxn) await client.query('ROLLBACK').catch(() => {});
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_affected: affected, errors, error_sample: sample, batches: Math.ceil(iterations / chunk) };
}

async function batchUpdate(client, kase, rows, iterations, chunk) {
  const names = kase.params || [];
  const lat = []; let affected = 0; let errors = 0; let sample = '';
  const start = nowMs();
  try {
    for (let s = 0; s < iterations; s += chunk) {
      const n = Math.min(chunk, iterations - s);
      const tuples = [];
      for (let r = 0; r < n; r += 1) {
        tuples.push(`($${r * 2 + 1}::bigint,$${r * 2 + 2}::text)`);
      }
      const sql = kase.sql + tuples.join(',') + ') AS v(id, payload) WHERE bench_writes.id = v.id';
      const t0 = nowMs();
      const result = await client.query({ text: sql, values: chunkValues(names, s, n, rows) });
      if (result.rowCount > 0) affected += result.rowCount;
      lat.push(nowMs() - t0);
    }
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_affected: affected, errors, error_sample: sample, batches: Math.ceil(iterations / chunk) };
}

async function batchDelete(client, kase, rows, iterations, chunk) {
  const lat = []; let affected = 0; let errors = 0; let sample = '';
  const start = nowMs();
  try {
    for (let s = 0; s < iterations; s += chunk) {
      const n = Math.min(chunk, iterations - s);
      const t0 = nowMs();
      const result = await client.query({
        name: statementName(kase.sql), text: kase.sql, values: [s + 1, s + n],
      });
      if (result.rowCount > 0) affected += result.rowCount;
      lat.push(nowMs() - t0);
    }
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_affected: affected, errors, error_sample: sample, batches: Math.ceil(iterations / chunk) };
}

async function txnLoop(client, kase, rows, iterations) {
  const names = kase.params || [];
  const lat = []; let affected = 0; let errors = 0; let sample = '';
  const start = nowMs();
  try {
    await client.query('BEGIN');
    for (let i = 0; i < iterations; i += 1) {
      const t0 = nowMs();
      const result = await client.query({ name: statementName(kase.sql), text: kase.sql, values: paramsFor(names, i, rows) });
      if (result.rowCount > 0) affected += result.rowCount;
      lat.push(nowMs() - t0);
    }
    await client.query('COMMIT');
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
    await client.query('ROLLBACK').catch(() => {});
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), rows_affected: affected, errors, error_sample: sample, transactions: 1 };
}

const INSERT_ROW = 'INSERT INTO bench_writes (id, k, payload, tag) VALUES ($1,$2,$3,$4)';
const rowValues = (i, rows) => [i + 1, (i * 37) % rows, 'payload-' + i, 't' + (i % 10)];

async function txnCase(client, kase, rows, iterations) {
  const perTxn = kase.per_txn || 1;
  const scenario = kase.scenario;
  const lat = []; let txns = 0; let errors = 0; let affected = 0; let sample = '';
  const start = nowMs();
  try {
    for (let i = 0; i < iterations; i += 1) {
      const t0 = nowMs();
      await client.query('BEGIN');
      if (scenario === 'commit' || scenario === 'rollback') {
        await client.query(INSERT_ROW, rowValues(i, rows));
      } else if (scenario === 'multi') {
        for (let j = 0; j < perTxn; j += 1) await client.query(INSERT_ROW, rowValues(i * perTxn + j, rows));
      } else if (scenario === 'constraint' || scenario === 'recovery') {
        await client.query(INSERT_ROW, rowValues(i, rows));
        try {
          await client.query(INSERT_ROW, rowValues(i, rows));
        } catch (err) {
          if (err.code !== '23505') throw err;
          errors += 1;
          await client.query('ROLLBACK');
          if (scenario === 'recovery') {
            await client.query('BEGIN');
            await client.query(INSERT_ROW, rowValues(1000000 + i, rows));
            await client.query('COMMIT');
            txns += 1; affected += 1;
          }
          lat.push(nowMs() - t0);
          continue;
        }
      }
      if (scenario === 'rollback') await client.query('ROLLBACK');
      else await client.query('COMMIT');
      txns += 1;
      lat.push(nowMs() - t0);
    }
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
    await client.query('ROLLBACK').catch(() => {});
  }
  const wall = nowMs() - start;
  return { ...summarise(lat, wall), transactions: txns, rows_affected: affected, errors, error_sample: sample };
}

const APP_UPDATE = 'UPDATE products SET stock = stock - $1 WHERE id = $2 AND stock >= $3';
const APP_INSERT = 'INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status) VALUES ($1,$2,$3,$4,$5::text::numeric,$6)';

async function appOrderTxn(client, kase, rows, iterations) {
  const lat = []; let errors = 0; let committed = 0; let rolled = 0; let sample = '';
  const start = nowMs();
  try {
    for (let i = 0; i < iterations; i += 1) {
      const product = ((i * 7) % 1000) + 1;
      const t0 = nowMs();
      await client.query('BEGIN');
      const updated = await client.query(APP_UPDATE, [1, product, 1]);
      if (updated.rowCount === 1) {
        await client.query(APP_INSERT, [2000000 + i, product, product, 1, '19.99', 'new']);
        await client.query('COMMIT');
        committed += 1;
      } else {
        await client.query('ROLLBACK');
        rolled += 1;
      }
      lat.push(nowMs() - t0);
    }
  } catch (err) {
    errors += 1;
    sample = firstLine(err);
    await client.query('ROLLBACK').catch(() => {});
  }
  const wall = nowMs() - start;
  return {
    ...summarise(lat, wall), transactions: committed + rolled, rows_affected: committed,
    rolled_back: rolled, errors, error_sample: sample,
  };
}

function poolConfig(args, size) {
  return {
    host: args.host,
    port: args.port,
    user: args.user,
    password: args.password,
    database: args.db,
    max: size,
    application_name: 'kode-bench-' + LANGUAGE,
    connectionTimeoutMillis: 60000,
  };
}

async function conc(args, kase, rows, ops) {
  const mode = kase.mode;
  const workers = kase.workers || 10;
  const lat = []; let errors = 0; let deadlocks = 0; let sample = '';

  const setupStart = nowMs();
  const pool = new Pool(poolConfig(args, workers));
  const connections = [];
  await Promise.all(Array.from({ length: workers }, async () => {
    connections.push(await pool.connect());
  }));
  const setupMs = nowMs() - setupStart;

  const work = async (worker) => {
    const client = connections[worker];
    for (let j = 0; j < ops; j += 1) {
      const write = mode === 'write' || (mode === 'mixed' && j % 10 >= 7);
      const t0 = nowMs();
      try {
        if (write) {
          await client.query(INSERT_ROW, [worker * 1000000 + j + 1, (j * 37) % Math.max(rows, 1), 'payload-w' + worker, 't' + (j % 10)]);
        } else {
          await client.query({ name: statementName('pk'), text: 'SELECT id, payload FROM bench_events WHERE id = $1', values: [(((worker * ops + j) * 7) % rows) + 1] });
        }
      } catch (err) {
        errors += 1;
        if (isDeadlock(err)) deadlocks += 1;
        sample = sample || firstLine(err);
      }
      lat.push(nowMs() - t0);
    }
  };

  const start = nowMs();
  await Promise.all(Array.from({ length: workers }, (_, w) => work(w)));
  const wall = nowMs() - start;
  connections.forEach((c) => c.release());
  await pool.end();
  return {
    ...summarise(lat, wall), errors, deadlocks, workers, connections: workers,
    setup_ms: round(setupMs, 3), error_sample: sample,
  };
}

async function conn(args, kase, rows, iterations) {
  const scenario = kase.scenario;
  const lat = []; let errors = 0; let sample = ''; let setupMs = 0; let connections = 0;
  const config = poolConfig(args, 1);
  delete config.max;

  if (scenario === 'connect') {
    for (let i = 0; i < iterations; i += 1) {
      const t0 = nowMs();
      try {
        const client = new Client(config);
        await client.connect();
        await client.query('SELECT 1');
        await client.end();
      } catch (err) { errors += 1; sample = sample || firstLine(err); }
      lat.push(nowMs() - t0);
    }
    connections = 1;
  } else if (scenario === 'reuse') {
    const t0 = nowMs();
    const client = new Client(config);
    await client.connect();
    setupMs = nowMs() - t0;
    for (let i = 0; i < iterations; i += 1) {
      const t = nowMs();
      try {
        await client.query({ name: statementName('pk'), text: 'SELECT id, payload FROM bench_events WHERE id = $1', values: [(((i * 7) % rows) + 1)] });
      } catch (err) { errors += 1; sample = sample || firstLine(err); }
      lat.push(nowMs() - t);
    }
    await client.end();
    connections = 1;
  } else if (scenario === 'pool') {
    const t0 = nowMs();
    const pool = new Pool(poolConfig(args, 10));
    const warm = await Promise.all(Array.from({ length: 10 }, () => pool.connect()));
    warm.forEach((c) => c.release());
    setupMs = nowMs() - t0;
    for (let i = 0; i < iterations; i += 1) {
      const t = nowMs();
      try {
        await pool.query({ name: statementName('pk'), text: 'SELECT id, payload FROM bench_events WHERE id = $1', values: [(((i * 7) % rows) + 1)] });
      } catch (err) { errors += 1; sample = sample || firstLine(err); }
      lat.push(nowMs() - t);
    }
    await pool.end();
    connections = 10;
  } else if (scenario === 'exhaust') {
    const t0 = nowMs();
    const pool = new Pool(poolConfig(args, 4));
    const warm = await Promise.all(Array.from({ length: 4 }, () => pool.connect()));
    warm.forEach((c) => c.release());
    setupMs = nowMs() - t0;
    const work = async (w) => {
      for (let j = 0; j < iterations; j += 1) {
        const t = nowMs();
        try {
          await pool.query({ name: statementName('pk_short'), text: 'SELECT id FROM bench_events WHERE id = $1', values: [(((w * iterations + j) % rows) + 1)] });
        } catch (err) { errors += 1; sample = sample || firstLine(err); }
        lat.push(nowMs() - t);
      }
    };
    await Promise.all(Array.from({ length: 20 }, (_, w) => work(w)));
    await pool.end();
    connections = 4;
  } else if (scenario === 'recover') {
    const control = new Client(config);
    await control.connect();
    for (let i = 0; i < iterations; i += 1) {
      const t0 = nowMs();
      try {
        const victim = new Client(config);
        await victim.connect();
        const pid = (await victim.query('SELECT pg_backend_pid()')).rows[0].pg_backend_pid;
        await control.query('SELECT pg_terminate_backend($1)', [pid]);
        try {
          await victim.query('SELECT 1');
        } catch (err) { errors += 1; }
        await victim.end().catch(() => {});
        await control.query('SELECT count(*) FROM bench_events WHERE id = 1');
      } catch (err) { errors += 1; sample = sample || firstLine(err); }
      lat.push(nowMs() - t0);
    }
    await control.end();
    connections = 2;
  } else {
    throw new Error('unknown connection scenario: ' + scenario);
  }

  const wall = lat.reduce((a, b) => a + b, 0);
  return {
    ...summarise(lat, wall), errors, error_sample: sample, setup_ms: round(setupMs, 3),
    connections, reconnect_ok: scenario === 'recover' && errors === iterations,
  };
}

async function runChecks(client, kase, rows) {
  const out = [];
  for (const check of kase.checks || []) {
    const values = paramsFor(check.params || [], 0, rows);
    // rowMode 'array' keeps column order and survives duplicate column names,
    // which a plain row object would collapse into one key.
    const result = await client.query({ text: check.sql, values, rowMode: 'array' });
    const row = result.rows[0] || [];
    out.push(Array.from(row).map((value) => render(value)));
  }
  return out;
}

function render(value) {
  if (value === null || value === undefined) return 'NULL';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  return String(value);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const spec = JSON.parse(fs.readFileSync(args.cases, 'utf8'));
  const chunk = spec.chunk || 500;
  let cases = spec.cases.filter((c) => c.workload === args.workload);
  if (args.only) {
    const wanted = new Set(args.only.split(','));
    cases = cases.filter((c) => wanted.has(c.id));
  }
  if (cases.length === 0) {
    process.stdout.write(JSON.stringify({ language: LANGUAGE, workload: args.workload, error: 'no cases' }) + '\n');
    return 2;
  }

  const config = poolConfig(args, 1);
  delete config.max;
  const client = new Client(config);
  await client.connect();

  for (const kase of cases) {
    let iterations = kase.iterations === 'rows' ? args.rows : kase.iterations;
    if (kase.setup) {
      await client.query(kase.setup.replace('{rows}', String(args.rows)));
    }
    const startedCpu = cpuMs();
    const t0 = nowMs();
    let result;
    switch (kase.shape) {
      case 'stmt_loop': result = await stmtLoop(client, kase, args.rows, iterations); break;
      case 'query_loop': result = await queryLoop(client, kase, args.rows, iterations); break;
      case 'chunk_insert': result = await chunkInsert(client, kase, args.rows, iterations, chunk, false); break;
      case 'chunk_insert_txn': result = await chunkInsert(client, kase, args.rows, iterations, chunk, true); break;
      case 'batch_update': result = await batchUpdate(client, kase, args.rows, iterations, chunk); break;
      case 'batch_delete': result = await batchDelete(client, kase, args.rows, iterations, chunk); break;
      case 'txn_loop': result = await txnLoop(client, kase, args.rows, iterations); break;
      case 'txn_case': result = await txnCase(client, kase, args.rows, iterations); break;
      case 'app_order_txn': result = await appOrderTxn(client, kase, args.rows, iterations); break;
      case 'conc':
        result = await conc(args, kase, args.rows, args.ops);
        iterations = args.ops * kase.workers;
        break;
      case 'conn': result = await conn(args, kase, args.rows, iterations); break;
      default: throw new Error('unknown shape: ' + kase.shape);
    }
    const caseCpu = cpuMs() - startedCpu;
    const checks = await runChecks(client, kase, args.rows);

    const record = {
      language: LANGUAGE,
      workload: args.workload,
      case: kase.id,
      shape: kase.shape,
      rows: args.rows,
      iterations,
      expect_errors: Boolean(kase.expect_errors),
      checks,
      cpu_ms: round(caseCpu, 3),
      peak_rss_kb: peakRssKb(),
      elapsed_ms: round(nowMs() - t0, 3),
      setup_ms: result.setup_ms || 0,
      error_sample: result.error_sample || '',
      status: 'ok',
      ...result,
    };
    if (record.throughput === undefined) {
      record.throughput = record.wall_ms > 0 ? round(record.ops / (record.wall_ms / 1000), 2) : 0;
    }
    process.stdout.write(JSON.stringify(record) + '\n');
  }

  await client.end();
  return 0;
}

main().then((code) => process.exit(code)).catch((err) => {
  process.stderr.write(String(err && err.stack ? err.stack : err) + '\n');
  process.exit(1);
});
