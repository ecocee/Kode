//! B3: lists, maps and JSON through the LLVM backend.
//!
//! Each program is built and run with `KODE_LEAK_CHECK=1`; the expected output
//! is the expected runtime output for the same source. The inline fast
//! paths (len/get/set/push) carry the ownership rules of the runtime calls they
//! replace, so a bad retain/release or a missing bounds check fails here.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_kode-llvm");

fn have_clang() -> bool {
    Command::new(kode_llvm::clang_program())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_file(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kode-llvm-col-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(format!("{name}.kode"));
    std::fs::write(&path, src).expect("write program");
    path
}

struct Run {
    stdout: String,
    stderr: String,
    ok: bool,
}

fn run(name: &str, src: &str) -> Run {
    let path = temp_file(name, src);
    let dir = path.parent().unwrap().to_path_buf();
    let exe = dir.join("program");
    let built = Command::new(BIN)
        .arg("build")
        .arg(&path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("build with kode-llvm");
    if !built.status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        panic!(
            "`{name}` did not build:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
    }
    let out = Command::new(&exe)
        .env("KODE_LEAK_CHECK", "1")
        .output()
        .expect("run the built program");
    let _ = std::fs::remove_dir_all(&dir);
    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        ok: out.status.success(),
    }
}

fn check(name: &str, src: &str, expected: &str) {
    if !have_clang() {
        eprintln!("skipping `{name}`: no clang on PATH");
        return;
    }
    let r = run(name, src);
    assert!(r.ok, "`{name}` failed:\n{}", r.stderr);
    assert_eq!(r.stdout.trim(), expected.trim(), "`{name}` output");
    assert!(
        r.stderr.trim().is_empty(),
        "`{name}` reported runtime problems (leaks?):\n{}",
        r.stderr
    );
}

#[test]
fn int_lists_push_get_set_len_iterate_and_slice() {
    let src = "\
xs: [int] = []
for i in 0..10 {
    xs.add(i * i)
}
xs[3] = 99
print(xs.len(), xs[0], xs[3], xs[9])
print(xs)
total = 0
for x in xs {
    total += x
}
print(total)
print(xs[2..5])
print(xs[1..xs.len()])
";
    check(
        "intlist",
        src,
        "10 0 99 81
[0, 1, 4, 99, 16, 25, 36, 49, 64, 81]
375
[4, 99, 16]
[1, 4, 99, 16, 25, 36, 49, 64, 81]",
    );
}

#[test]
fn packed_bool_lists_use_one_byte_items() {
    let src = "\
flags: [bool] = []
for i in 0..100 {
    flags.add(true)
}
flags[5] = false
count = 0
for f in flags {
    if f {
        count += 1
    }
}
print(flags.len(), count, flags[5], flags[6])
";
    check("boollist", src, "100 99 false true");
}

#[test]
fn string_lists_replace_items_through_the_runtime() {
    let src = "\
names: [string] = []
names.add(\"asha\")
names.add(\"ravi\")
names[1] = \"zed\"
print(names, names[1], names.len())
i = 0
while i < 2000 {
    names[0] = \"n${i}\"
    i += 1
}
print(names[0], names.len())
";
    check("strlist", src, "[\"asha\", \"zed\"] zed 2\nn1999 2");
}

#[test]
fn an_out_of_range_index_uses_the_runtime_message() {
    if !have_clang() {
        eprintln!("skipping `oob`: no clang on PATH");
        return;
    }
    let r = run(
        "oob",
        "\
xs = [1, 2]
print(xs[5])
",
    );
    assert!(!r.ok, "an out-of-range index must stop the program");
    assert!(
        r.stderr.contains("out of range") && r.stderr.contains("length 2"),
        "expected the runtime's bounds message, got: {}",
        r.stderr
    );
}

#[test]
fn maps_insert_lookup_and_iterate() {
    let src = "\
ages: {string: int} = {}
ages[\"asha\"] = 30
ages[\"ravi\"] = 25
print(ages.len(), ages[\"asha\"], ages.has(\"zed\"))
keys = ages.keys()
keys.sort()
for k in keys {
    print(k, ages[k])
}
";
    check("maps", src, "2 30 false\nasha 30\nravi 25");
}

#[test]
fn nested_collections_print_expected() {
    let src = "\
rows = [[1, 2], [3]]
print(rows)
notes = {\"tags\": [\"ai\"]}
print(notes)
print(rows[0][1], notes[\"tags\"][0])
";
    check("nested", src, "[[1, 2], [3]]\n{\"tags\": [\"ai\"]}\n2 ai");
}

#[test]
fn json_parse_index_and_encode() {
    let src = "\
doc = json_parse(\"{\\\"name\\\":\\\"Asha\\\",\\\"tags\\\":[\\\"ai\\\",\\\"kode\\\"]}\")
print(doc[\"name\"].as_string())
print(doc[\"tags\"].len(), doc[\"tags\"][1].as_string())
print(doc[\"tags\"].encode())
built = json_obj()
built[\"n\"] = json_int(7)
built[\"ok\"] = json_bool(true)
print(json_map(built).encode())
";
    check(
        "json",
        src,
        "Asha
2 kode
[\"ai\",\"kode\"]
{\"n\":7,\"ok\":true}",
    );
}
