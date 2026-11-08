//! PostgreSQL client: `pg.connect`, `pg.exec`, `pg.query`, `pg.close`,
//! and connection pooling (`pg.pool`, `pg.pool_close`, `pg.pool_connect`,
//! `pg.pool_release`, `pg.pool_exec`, `pg.pool_query`).
//!
//! A connection is an `int` handle into a table of open connections, so a Kode
//! program can pass it around, keep several open, or share one between tasks
//! (each call locks only its own connection). The protocol lives in
//! `pg_wire.rs`; this file turns its results into JSON:
//!
//! ```text
//! { "command": "SELECT", "columns": ["id"], "rows": [{"id": 1}], "affected": 1 }
//! ```
//!
//! Column values keep their PostgreSQL type where JSON has one (bool, int,
//! float) and are strings otherwise, so `as_int`, `as_string` and `is_null`
//! behave the way a driver user expects.
//!
//! ## Connection pooling
//!
//! A pool is an `int` handle that holds a configurable number of connections.
//! `pg.pool_query` and `pg.pool_exec` borrow a connection, run the query, and
//! return the connection to the pool automatically. For explicit control, use
//! `pg.pool_connect` / `pg.pool_release`.
//!
//! Pool handles and connection handles are separate: closing a pool closes its
//! connections; closing a connection removes it from its pool.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use crate::error::throw_msg;
use crate::json::{
    bool_of, float_of, int_of, kind_of, make_bool, make_float, make_int, make_null, make_string,
    string_of, wrap_list, wrap_map, BOOL, FLOAT, INT, NULL, STR,
};
use crate::list::{kode_list_get, kode_list_len, kode_list_new, kode_list_push};
use crate::map::{kode_map_new, kode_map_set};
use crate::object::{kode_release, Header};
use crate::pg_wire::{Conn, Outcome};
use crate::string::{bytes, from_bytes};

type Shared = Arc<Mutex<Conn>>;

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);
static NEXT_POOL: AtomicI64 = AtomicI64::new(1);

static OPEN: OnceLock<Mutex<HashMap<i64, Shared>>> = OnceLock::new();
static POOLS: OnceLock<Mutex<HashMap<i64, Pool>>> = OnceLock::new();
static POOL_FOR_CONN: OnceLock<Mutex<HashMap<i64, i64>>> = OnceLock::new();

fn connections() -> &'static Mutex<HashMap<i64, Shared>> {
    OPEN.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pools() -> &'static Mutex<HashMap<i64, Pool>> {
    POOLS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pool_for_conn() -> &'static Mutex<HashMap<i64, i64>> {
    POOL_FOR_CONN.get_or_init(|| Mutex::new(HashMap::new()))
}

/// A connection pool with configurable size and idle timeout.
struct Pool {
    url: String,
    min_connections: i64,
    max_connections: i64,
    idle_timeout_ms: i64,
    connections: HashMap<i64, PoolSlot>,
}

struct PoolSlot {
    conn: Option<Shared>,
    acquired_at: Option<Instant>,
}

impl Pool {
    fn new(url: String, min_connections: i64, max_connections: i64, idle_timeout_ms: i64) -> Self {
        let min = if min_connections < 0 {
            0
        } else {
            min_connections
        };
        let max = if max_connections < 1 {
            1
        } else {
            max_connections
        };
        let idle = if idle_timeout_ms < 0 {
            30_000
        } else {
            idle_timeout_ms
        };
        Pool {
            url,
            min_connections: min,
            max_connections: max,
            idle_timeout_ms: idle,
            connections: HashMap::new(),
        }
    }

    fn fill_to_min(&mut self) -> Result<(), String> {
        let target = self.min_connections.max(1);
        while self.alive_count() < target as usize {
            if self.connections.len() >= self.max_connections as usize {
                break;
            }
            self.add_connection()?;
        }
        Ok(())
    }

    fn add_connection(&mut self) -> Result<i64, String> {
        let conn = Conn::open(&self.url)?;
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        let slot = PoolSlot {
            conn: Some(Arc::new(Mutex::new(conn))),
            acquired_at: None,
        };
        self.connections.insert(handle, slot);
        // Track which pool this connection belongs to
        lock(pool_for_conn()).insert(handle, /* pool_handle */ 0); // Will be set by caller
        Ok(handle)
    }

    fn alive_count(&self) -> usize {
        self.connections
            .values()
            .filter(|s| s.conn.is_some())
            .count()
    }

    fn prune_idle(&mut self) {
        let timeout = Duration::from_millis(self.idle_timeout_ms as u64);
        let mut to_remove = Vec::new();
        for (&handle, slot) in self.connections.iter() {
            if let Some(acquired) = slot.acquired_at {
                if acquired.elapsed() > timeout {
                    to_remove.push(handle);
                }
            }
        }
        for handle in to_remove {
            self.connections.remove(&handle);
        }
    }

    fn acquire(&mut self) -> Result<i64, String> {
        self.prune_idle();

        // Find an idle connection first
        for (&handle, slot) in self.connections.iter_mut() {
            if slot.conn.is_some() && slot.acquired_at.is_none() {
                slot.acquired_at = Some(Instant::now());
                return Ok(handle);
            }
        }

        // Create a new connection if under max
        if self.connections.len() < self.max_connections as usize {
            let handle = self.add_connection()?;
            if let Some(slot) = self.connections.get_mut(&handle) {
                slot.acquired_at = Some(Instant::now());
            }
            return Ok(handle);
        }

        Err("connection pool is at maximum capacity".into())
    }

    fn release(&mut self, handle: i64) {
        if let Some(slot) = self.connections.get_mut(&handle) {
            slot.acquired_at = None;
        }
    }

    fn remove_conn(&mut self, handle: i64) {
        self.connections.remove(&handle);
    }

    fn close_all(&mut self) {
        for slot in self.connections.values_mut() {
            if let Some(conn) = slot.conn.take() {
                lock(&conn).close();
            }
        }
        self.connections.clear();
    }
}

/// A poisoned connection mutex still holds a usable `Conn`; report the original
/// panics at their own site instead of turning every later query into a panic.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn shared(handle: i64) -> Result<Shared, String> {
    lock(connections())
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("connection {handle} is not open: it was closed or never opened"))
}

fn pool_slot(pool_handle: i64, conn_handle: i64) -> Result<(Arc<Mutex<Conn>>, i64), String> {
    let mut pool_guard = lock(pools());
    let pool = pool_guard
        .get_mut(&pool_handle)
        .ok_or_else(|| format!("pool {pool_handle} is not open: it was closed or never opened"))?;

    let slot = pool
        .connections
        .get_mut(&conn_handle)
        .ok_or_else(|| format!("connection {conn_handle} is not part of pool {pool_handle}"))?;

    let conn = slot.conn.clone();
    if conn.is_none() {
        return Err(format!(
            "connection {conn_handle} in pool {pool_handle} is closed"
        ));
    }

    Ok((conn.unwrap(), conn_handle))
}

/// # Safety
/// `p` must be a valid string.
unsafe fn text<'a>(p: *const Header) -> &'a str {
    std::str::from_utf8_unchecked(bytes(p))
}

/// `pg.connect(url)` → a connection handle.
///
/// # Safety
/// `url` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_connect(url: *const Header) -> i64 {
    match Conn::open(text(url)) {
        Ok(conn) => {
            let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
            lock(connections()).insert(handle, Arc::new(Mutex::new(conn)));
            handle
        }
        Err(message) => {
            throw_msg(&message);
            0
        }
    }
}

/// `pg.pool(url, min_connections, max_connections, idle_timeout_ms)` → a pool handle.
///
/// Creates a connection pool with the given configuration. The pool pre-creates
/// `min_connections` connections and can grow up to `max_connections`.
/// Idle connections are pruned after `idle_timeout_ms` milliseconds.
///
/// # Safety
/// `url` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_pool(
    url: *const Header,
    min_connections: i64,
    max_connections: i64,
    idle_timeout_ms: i64,
) -> i64 {
    let url_str = text(url);
    let mut pool = Pool::new(
        url_str.to_string(),
        min_connections,
        max_connections,
        idle_timeout_ms,
    );
    match pool.fill_to_min() {
        Ok(_) => {
            let handle = NEXT_POOL.fetch_add(1, Ordering::Relaxed);
            lock(pools()).insert(handle, pool);
            handle
        }
        Err(message) => {
            throw_msg(&message);
            0
        }
    }
}

/// `pg.pool_close(pool)` closes all connections in the pool.
///
/// # Safety
/// `pool` must be a valid pool handle.
#[no_mangle]
pub extern "C" fn kode_pg_pool_close(pool_handle: i64) {
    if let Some(mut pool) = lock(pools()).remove(&pool_handle) {
        // Remove all connections from the global connections table and pool-for-conn mapping
        let mut conns = lock(connections());
        let mut pool_conns = lock(pool_for_conn());
        for &conn_handle in pool.connections.keys() {
            conns.remove(&conn_handle);
            pool_conns.remove(&conn_handle);
        }
        pool.close_all();
    }
}

/// `pg.pool_connect(pool)` → a connection handle borrowed from the pool.
///
/// Returns a connection from the pool, creating one if needed (up to max).
/// The connection must be returned with `pg.pool_release` when done.
///
/// # Safety
/// `pool` must be a valid pool handle.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_pool_connect(pool_handle: i64) -> i64 {
    let mut pool_guard = lock(pools());
    let pool = match pool_guard.get_mut(&pool_handle) {
        Some(pool) => pool,
        None => {
            throw_msg(&format!("pool {pool_handle} is not open"));
            return 0;
        }
    };

    match pool.acquire() {
        Ok(conn_handle) => {
            // Record that this connection belongs to this pool
            lock(pool_for_conn()).insert(conn_handle, pool_handle);
            conn_handle
        }
        Err(message) => {
            throw_msg(&message);
            0
        }
    }
}

/// `pg.pool_release(conn)` returns a connection to the pool.
///
/// # Safety
/// `conn` must be a valid connection handle obtained from `pg.pool_connect`.
#[no_mangle]
pub extern "C" fn kode_pg_pool_release(conn_handle: i64) {
    // Find the pool this connection belongs to and release it
    if let Some(pool_handle) = lock(pool_for_conn()).get(&conn_handle).cloned() {
        if let Some(pool) = lock(pools()).get_mut(&pool_handle) {
            pool.release(conn_handle);
        }
    }
    // Also remove from global connections if it was there (shouldn't be for pool connections)
    lock(connections()).remove(&conn_handle);
}

/// `pg.pool_exec(pool, sql)` → the last result set of a simple query.
///
/// Borrows a connection from the pool, executes the query, and returns the
/// connection to the pool automatically.
///
/// # Safety
/// `pool` must be a valid pool handle and `sql` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_pool_exec(pool_handle: i64, sql: *const Header) -> *mut Header {
    let conn_handle = kode_pg_pool_connect(pool_handle);
    if conn_handle == 0 {
        return std::ptr::null_mut();
    }

    let result = kode_pg_exec_via_pool(conn_handle, sql);
    kode_pg_pool_release(conn_handle);
    result
}

/// `pg.pool_query(pool, sql, params)` → the result of one prepared statement.
///
/// Borrows a connection from the pool, executes the query, and returns the
/// connection to the pool automatically.
///
/// # Safety
/// `pool` must be a valid pool handle, `sql` a valid string, and `params` a list of JSON values.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_pool_query(
    pool_handle: i64,
    sql: *const Header,
    params: *mut Header,
) -> *mut Header {
    let conn_handle = kode_pg_pool_connect(pool_handle);
    if conn_handle == 0 {
        return std::ptr::null_mut();
    }

    let result = kode_pg_query_via_pool(conn_handle, sql, params);
    kode_pg_pool_release(conn_handle);
    result
}

/// `pg.exec` for a pool connection - looks up connection in pool directly.
///
/// # Safety
/// `conn_handle` must be a valid connection handle from a pool.
unsafe extern "C" fn kode_pg_exec_via_pool(handle: i64, sql: *const Header) -> *mut Header {
    // Look up the connection in the pool
    let (conn, _pool_handle) = match pool_slot_for_query(handle) {
        Ok(result) => result,
        Err(message) => {
            throw_msg(&message);
            return std::ptr::null_mut();
        }
    };
    let outcome = {
        let mut guard = lock(&conn);
        guard.simple(text(sql))
    };
    match outcome {
        Ok(outcome) => outcome_json(&outcome),
        Err(message) => {
            throw_msg(&message);
            std::ptr::null_mut()
        }
    }
}

/// `pg.query` for a pool connection - looks up connection in pool directly.
///
/// # Safety
/// `conn_handle` must be a valid connection handle from a pool.
unsafe extern "C" fn kode_pg_query_via_pool(
    handle: i64,
    sql: *const Header,
    params: *mut Header,
) -> *mut Header {
    // Look up the connection in the pool
    let (conn, _pool_handle) = match pool_slot_for_query(handle) {
        Ok(result) => result,
        Err(message) => {
            throw_msg(&message);
            return std::ptr::null_mut();
        }
    };

    let count = kode_list_len(params);
    let mut bound: Vec<Option<Vec<u8>>> = Vec::with_capacity(count as usize);
    for i in 0..count {
        let raw = kode_list_get(params, i, 0) as *mut Header;
        match parameter(raw) {
            Ok(value) => bound.push(value),
            Err(message) => {
                throw_msg(&message);
                return std::ptr::null_mut();
            }
        }
    }

    let outcome = {
        let mut guard = lock(&conn);
        guard.query(text(sql), &bound)
    };
    match outcome {
        Ok(outcome) => outcome_json(&outcome),
        Err(message) => {
            throw_msg(&message);
            std::ptr::null_mut()
        }
    }
}

/// Look up a connection handle in its pool and return the connection.
/// Returns (conn, pool_handle).
fn pool_slot_for_query(conn_handle: i64) -> Result<(Arc<Mutex<Conn>>, i64), String> {
    let pool_handle = lock(pool_for_conn())
        .get(&conn_handle)
        .cloned()
        .ok_or_else(|| {
            format!("connection {conn_handle} is not open: it was closed or never opened")
        })?;

    let mut pool_guard = lock(pools());
    let pool = pool_guard
        .get_mut(&pool_handle)
        .ok_or_else(|| format!("pool {pool_handle} is not open"))?;

    let slot = pool
        .connections
        .get_mut(&conn_handle)
        .ok_or_else(|| format!("connection {conn_handle} not found in pool"))?;

    let conn = slot.conn.clone();
    if conn.is_none() {
        return Err(format!("connection {conn_handle} in pool is closed"));
    }

    Ok((conn.unwrap(), pool_handle))
}

/// Check if a connection is from a pool and remove it from the pool when closing.
fn remove_from_pool_if_needed(handle: i64) {
    if let Some(pool_handle) = lock(pool_for_conn()).remove(&handle) {
        if let Some(pool) = lock(pools()).get_mut(&pool_handle) {
            pool.remove_conn(handle);
        }
    }
}

/// `pg.exec(conn, sql)` → the last result set of a simple query.
///
/// # Safety
/// `sql` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_exec(handle: i64, sql: *const Header) -> *mut Header {
    // Try global connections first, then check pools
    let conn = match shared(handle) {
        Ok(conn) => conn,
        Err(_) => {
            // Not in global connections, try to find in a pool
            match pool_slot_for_query(handle) {
                Ok((conn, _)) => conn,
                Err(message) => {
                    throw_msg(&message);
                    return std::ptr::null_mut();
                }
            }
        }
    };
    let outcome = {
        let mut guard = lock(&conn);
        guard.simple(text(sql))
    };
    match outcome {
        Ok(outcome) => outcome_json(&outcome),
        Err(message) => {
            throw_msg(&message);
            std::ptr::null_mut()
        }
    }
}

/// `pg.query(conn, sql, params)` → the result of one prepared statement.
///
/// # Safety
/// `sql` must be a valid string and `params` a list of JSON values.
#[no_mangle]
pub unsafe extern "C" fn kode_pg_query(
    handle: i64,
    sql: *const Header,
    params: *mut Header,
) -> *mut Header {
    // Try global connections first, then check pools
    let conn = match shared(handle) {
        Ok(conn) => conn,
        Err(_) => {
            // Not in global connections, try to find in a pool
            match pool_slot_for_query(handle) {
                Ok((conn, _)) => conn,
                Err(message) => {
                    throw_msg(&message);
                    return std::ptr::null_mut();
                }
            }
        }
    };

    let count = kode_list_len(params);
    let mut bound: Vec<Option<Vec<u8>>> = Vec::with_capacity(count as usize);
    for i in 0..count {
        let raw = kode_list_get(params, i, 0) as *mut Header;
        match parameter(raw) {
            Ok(value) => bound.push(value),
            Err(message) => {
                throw_msg(&message);
                return std::ptr::null_mut();
            }
        }
    }

    let outcome = {
        let mut guard = lock(&conn);
        guard.query(text(sql), &bound)
    };
    match outcome {
        Ok(outcome) => outcome_json(&outcome),
        Err(message) => {
            throw_msg(&message);
            std::ptr::null_mut()
        }
    }
}

/// `pg.close(conn)`. Closing twice, or closing an unknown handle, does nothing.
#[no_mangle]
pub extern "C" fn kode_pg_close(handle: i64) {
    // First check if this is a pool connection and remove from pool
    remove_from_pool_if_needed(handle);

    if let Some(conn) = lock(connections()).remove(&handle) {
        lock(&conn).close();
    }
}

/// A parameter is sent in PostgreSQL's text format: `json.null` binds NULL,
/// and a list or map is rejected rather than stringified into something that
/// silently matches no row.
unsafe fn parameter(value: *mut Header) -> Result<Option<Vec<u8>>, String> {
    if value.is_null() {
        return Ok(None);
    }
    match kind_of(value) {
        NULL => Ok(None),
        BOOL => Ok(Some(if bool_of(value) {
            b"true".to_vec()
        } else {
            b"false".to_vec()
        })),
        INT => Ok(Some(int_of(value).to_string().into_bytes())),
        FLOAT => Ok(Some(float_of(value).to_string().into_bytes())),
        STR => Ok(Some(bytes(string_of(value)).to_vec())),
        _ => Err("a query parameter must be null, a bool, an int, a float or a string".into()),
    }
}

/// PostgreSQL command tags end with the affected row count: `INSERT 0 5`,
/// `UPDATE 3`, `SELECT 12`. DDL tags such as `CREATE TABLE` have none.
fn affected(tag: &str) -> i64 {
    tag.split_whitespace()
        .last()
        .and_then(|word| word.parse::<i64>().ok())
        .unwrap_or(0)
}

unsafe fn cell(oid: u32, value: Option<&[u8]>) -> *mut Header {
    let raw = match value {
        None => return make_null(),
        Some(raw) => raw,
    };
    let text = match std::str::from_utf8(raw) {
        Ok(text) => text,
        Err(_) => return make_string(from_bytes(raw)),
    };
    match oid {
        16 => make_bool(text == "t" || text == "true"),
        20 | 21 | 23 | 26 => match text.parse::<i64>() {
            Ok(n) => make_int(n),
            Err(_) => make_string(from_bytes(raw)),
        },
        700 | 701 => match text.parse::<f64>() {
            Ok(n) => make_float(n),
            Err(_) => make_string(from_bytes(raw)),
        },
        _ => make_string(from_bytes(raw)),
    }
}

unsafe fn set(map: *mut Header, key: &str, value: *mut Header) {
    let key = from_bytes(key.as_bytes());
    kode_map_set(map, key, value as i64);
    kode_release(key);
}

/// Build the JSON object a `pg.exec` or `pg.query` call returns.
unsafe fn outcome_json(out: &Outcome) -> *mut Header {
    let result = kode_map_new(1);
    set(
        result,
        "command",
        make_string(from_bytes(out.tag.as_bytes())),
    );

    let columns = kode_list_new(1);
    for column in &out.columns {
        kode_list_push(
            columns,
            make_string(from_bytes(column.name.as_bytes())) as i64,
        );
    }
    set(result, "columns", wrap_list(columns));

    let rows = kode_list_new(1);
    for row in &out.rows {
        let object = kode_map_new(1);
        for (column, value) in out.columns.iter().zip(row.iter()) {
            let key = from_bytes(column.name.as_bytes());
            kode_map_set(object, key, cell(column.oid, value.as_deref()) as i64);
            kode_release(key);
        }
        kode_list_push(rows, wrap_map(object) as i64);
    }
    set(result, "rows", wrap_list(rows));
    set(result, "affected", make_int(affected(&out.tag)));

    if let Some(notice) = &out.notice {
        set(result, "notice", make_string(from_bytes(notice.as_bytes())));
    }
    wrap_map(result)
}

#[cfg(test)]
mod tests {
    use super::affected;
    use super::parameter;
    use crate::json::{make_bool, make_int, make_null, make_string};
    use crate::object::kode_release;
    use crate::string::from_bytes;

    #[test]
    fn command_tags() {
        assert_eq!(affected("INSERT 0 5"), 5);
        assert_eq!(affected("UPDATE 3"), 3);
        assert_eq!(affected("SELECT 12"), 12);
        assert_eq!(affected("DELETE 0"), 0);
        assert_eq!(affected("CREATE TABLE"), 0);
        assert_eq!(affected("BEGIN"), 0);
    }

    #[test]
    fn parameters_keep_their_type() {
        unsafe {
            let null = make_null();
            assert_eq!(parameter(null).expect("null"), None);
            kode_release(null);

            let int = make_int(42);
            assert_eq!(parameter(int).expect("int").as_deref(), Some(&b"42"[..]));
            kode_release(int);

            let flag = make_bool(true);
            assert_eq!(
                parameter(flag).expect("bool").as_deref(),
                Some(&b"true"[..])
            );
            kode_release(flag);

            let text = make_string(from_bytes("a b".as_bytes()));
            assert_eq!(
                parameter(text).expect("string").as_deref(),
                Some(&b"a b"[..])
            );
            kode_release(text);

            let list = crate::json::wrap_list(crate::list::kode_list_new(1));
            let error = parameter(list).expect_err("a list is not a parameter");
            assert!(error.contains("must be null, a bool"));
            kode_release(list);
        }
    }
}
