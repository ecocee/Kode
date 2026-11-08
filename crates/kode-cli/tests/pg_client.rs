//! Integration test for the PostgreSQL client (`pg.connect`, `pg.exec`,
//! `pg.query`, `pg.close`).
//!
//! It needs a real server, so it runs only when `KODE_TEST_PG_URL` is set:
//!
//! ```text
//! docker compose -f benchmarks/database/docker-compose.yml up -d postgres
//! KODE_TEST_PG_URL='postgres://bench:bench@127.0.0.1:55432/kode_bench?sslmode=disable' \
//!   cargo test -p kode-cli --test pg_client
//! ```
//!
//! The test creates and drops its own table, so it never touches benchmark data.

use std::process::Command;

#[test]
fn postgres_round_trip() {
    let url = match std::env::var("KODE_TEST_PG_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => {
            println!(
                "skipped: KODE_TEST_PG_URL is not set, so there is no PostgreSQL server to talk to"
            );
            return;
        }
    };
    let suffix = std::process::id().to_string();

    let program = r#"
url = args()[0]
suffix = args()[1]
table = "kode_pg_test_" + suffix
db = pg.connect(url)

pg.exec(db, "drop table if exists " + table)
print(pg.exec(db, "create table " + table + " (id int primary key, name text, note text)")["command"].as_string())

ins = pg.query(db, "insert into " + table + " values ($1, $2, $3), ($4, $5, $6)", [json.int(1), json.string("asha"), json.null(), json.int(2), json.string("ravi"), json.string("hi")])
print("insert", ins["command"].as_string(), ins["affected"].as_int())

sel = pg.query(db, "select id, name, note from " + table + " order by id", [])
rows = sel["rows"].as_list()
print("select", rows.len(), sel["columns"].as_list().len(), rows[0]["id"].as_int(), rows[0]["note"].is_null(), rows[1]["note"].as_string())

// A parameterised filter, then a row count from a command tag.
one = pg.query(db, "select name from " + table + " where id = $1", [json.int(2)])
print("filter", one["rows"].as_list()[0]["name"].as_string())
print("update", pg.query(db, "update " + table + " set name = $1", [json.string("meera")])["affected"].as_int())

// A failed statement is catchable, keeps the SQLSTATE, and leaves the
// connection usable.
pg.query(db, "select * from " + table + "_missing", []) catch e {
    print("error", e.message.contains("42P01"))
    json.parse("{}")
}
print("after error", pg.exec(db, "select count(*) as n from " + table)["rows"].as_list()[0]["n"].as_int())

// A rolled-back transaction leaves no rows behind.
pg.exec(db, "begin")
pg.exec(db, "insert into " + table + " values (3, 'ghost', null)")
pg.exec(db, "rollback")
print("rollback", pg.exec(db, "select count(*) as n from " + table)["rows"].as_list()[0]["n"].as_int())

// Text values that look like numbers stay text, and an int column stays an int.
typed = pg.exec(db, "select '007' as code, 7 as n, true as flag")
print("typed", typed["rows"].as_list()[0]["code"].as_string(), typed["rows"].as_list()[0]["n"].as_int(), typed["rows"].as_list()[0]["flag"].as_bool())

pg.exec(db, "drop table " + table)
pg.close(db)

pg.exec(db, "select 1") catch e {
    print("closed", e.message.contains("not open"))
    json.parse("{}")
}
"#;

    let path = std::env::temp_dir().join(format!("kode-pg-client-{}.kode", std::process::id()));
    std::fs::write(&path, program).expect("write the Kode program");
    let output = Command::new(env!("CARGO_BIN_EXE_kode"))
        .arg("run")
        .arg(&path)
        .arg("--")
        .arg(&url)
        .arg(&suffix)
        .env("KODE_LEAK_CHECK", "1")
        .output()
        .expect("run kode");
    let _ = std::fs::remove_file(&path);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the program failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout,
        "CREATE TABLE\n\
         insert INSERT 0 2 2\n\
         select 2 3 1 true hi\n\
         filter ravi\n\
         update 2\n\
         error true\n\
         after error 2\n\
         rollback 2\n\
         typed 007 7 true\n\
         closed true\n",
        "unexpected output\nstderr:\n{stderr}"
    );
}
