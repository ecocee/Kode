//! B4: classes, fields, methods, interfaces, generics and optional objects.
//!
//! Every program is built and run with `KODE_LEAK_CHECK=1`. The expected
//! strings are the output of the Cranelift backend for the same source, so a
//! layout, offset, retain/release or dispatch mistake shows up as a diff (or a
//! leak report) rather than a silent change.

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
    let dir = std::env::temp_dir().join(format!("kode-llvm-obj-{}-{}", std::process::id(), name));
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
fn fields_methods_and_defaults() {
    let src = "\
class Account {
    owner: string
    balance: int = 0
    rate: float = 1.5

    fn deposit(amount: int) {
        balance += amount
    }

    fn summary() -> string {
        return \"${owner}: ${balance}\"
    }

    fn rich() -> bool => balance > 100
}

a = Account(owner: \"Asha\")
a.deposit(50)
a.deposit(75)
print(a.summary(), a.rich(), a.rate)
a.balance = 10
a.owner = \"Ravi\"
print(a.summary(), a.rich())
";
    check("fields", src, "Asha: 125 true 1.5\nRavi: 10 false");
}

#[test]
fn nested_objects_and_collections_in_fields() {
    let src = "\
class Inner { n: int = 7 }
class Holder {
    inner: Inner
    items: [int] = []
    tags: {string: int} = {}
}
h = Holder(inner: Inner())
h.items.add(1)
h.items.add(2)
h.tags[\"a\"] = 5
print(h.inner.n, h.items.len(), h.items[1], h.tags[\"a\"])
h.inner.n = 9
print(h.inner.n)
";
    check("nested", src, "7 2 2 5\n9");
}

#[test]
fn object_references_alias() {
    let src = "\
class Box { v: string = \"x\" }
a = Box()
b = a
b.v = \"y\"
print(a.v, b.v, a == b)
c = Box()
c.v = \"z\"
print(a.v, c.v)
";
    check("alias", src, "y y true\ny z");
}

#[test]
fn early_returns_with_string_fields_stay_leak_free() {
    let src = "\
class Node {
    name: string
    next: Node? = none
}
fn build(n: int) -> Node {
    if n <= 0 {
        return Node(name: \"empty\")
    }
    root = Node(name: \"root ${n}\")
    return root
}
i = 0
while i < 2000 {
    x = build(i % 3)
    if len(x.name) == 0 {
        print(\"bad\")
    }
    i += 1
}
print(build(0).name, build(5).name)
";
    check("early", src, "empty root 5");
}

#[test]
fn optional_object_references() {
    let src = "\
class Point {
    x: int = 0
    y: int = 0
}
p: Point? = none
print(p == none)
q: Point? = Point(x: 3, y: 4)
print(q!.x, q!.y)
r = p ?? Point(x: 1, y: 2)
print(r.x)
";
    check("opt", src, "true\n3 4\n1");
}

#[test]
fn interface_calls_dispatch_by_class() {
    let src = "\
interface Shape {
    fn area() -> int
}
class Square implements Shape {
    side: int
    fn area() -> int => side * side
}
class Rect implements Shape {
    w: int
    h: int
    fn area() -> int => w * h
}
fn area_of(s: Shape) -> int {
    return s.area()
}
s: Shape = Square(3)
r: Shape = Rect(2, 5)
print(s.area(), r.area(), area_of(s), area_of(r))
";
    check("iface", src, "9 10 9 10");
}

#[test]
fn generic_functions_instantiate_for_objects() {
    let src = "\
class Pair {
    a: int
    b: int
}
fn first<T>(xs: [T]) -> T {
    return xs[0]
}
xs = [Pair(a: 1, b: 2), Pair(a: 3, b: 4)]
p = first(xs)
print(p.a, p.b)
";
    check("generic", src, "1 2");
}

#[test]
fn unwrapping_none_reports_the_line_and_hint() {
    if !have_clang() {
        eprintln!("skipping `unwrap`: no clang on PATH");
        return;
    }
    let r = run(
        "unwrap",
        "\
class P {
    x: int = 1
}
p: P? = none
print(p!.x)
",
    );
    assert!(!r.ok, "unwrapping none must stop the program");
    assert!(
        r.stderr.contains("`!` was used on a none value at line 5")
            && r.stderr.contains("hint:"),
        "expected the runtime's unwrap message, got: {}",
        r.stderr
    );
}
