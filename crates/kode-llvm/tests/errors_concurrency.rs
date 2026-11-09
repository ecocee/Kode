//! B5: error handling and concurrency.
//!
//! Errors are a pending-value protocol (`kode_err_throw` / `_pending` / `_take`)
//! plus ordinary control flow, so the B1 lowering should reproduce Cranelift
//! exactly. Tasks and channels are runtime calls with a function-pointer thunk;
//! they are concurrency, so every program runs under a wall-clock timeout and a
//! deadlock fails fast instead of hanging the suite.
//!
//! Expected strings and exit codes are what Cranelift prints for the same
//! source under `KODE_LEAK_CHECK=1`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_kode-llvm");
const TIMEOUT: Duration = Duration::from_secs(20);

struct Run {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

fn have_clang() -> bool {
    Command::new(kode_llvm::clang_program())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run to completion, or kill the process after `TIMEOUT` so a deadlock in a
/// task or channel cannot wedge the test binary.
fn run_with_timeout(exe: &Path) -> Run {
    let mut child = Command::new(exe)
        .env("KODE_LEAK_CHECK", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the built program");
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait().expect("poll the program") {
            Some(_) => break,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Run {
                    ok: false,
                    code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    timed_out: true,
                };
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    let out = child.wait_with_output().expect("collect output");
    Run {
        ok: out.status.success(),
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        timed_out: false,
    }
}

fn run(name: &str, src: &str) -> Run {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("kode-llvm-b5-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src_path = dir.join(format!("{name}.kode"));
    std::fs::write(&src_path, src).expect("write program");
    let exe = dir.join("program");
    let built = Command::new(BIN)
        .arg("build")
        .arg(&src_path)
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
    let r = run_with_timeout(&exe);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        !r.timed_out,
        "`{name}` did not finish within {TIMEOUT:?} (deadlock?)"
    );
    r
}

fn check(name: &str, src: &str, expected: &str) {
    if !have_clang() {
        eprintln!("skipping `{name}`: no clang on PATH");
        return;
    }
    let r = run(name, src);
    assert!(r.ok, "`{name}` failed (code {:?}): {}", r.code, r.stderr);
    assert_eq!(r.stdout.trim(), expected.trim(), "`{name}` output");
    assert!(
        r.stderr.trim().is_empty(),
        "`{name}` leaked or reported a runtime error:\n{}",
        r.stderr
    );
}

fn check_fails(name: &str, src: &str, expected: &str, stderr_has: &str) {
    if !have_clang() {
        eprintln!("skipping `{name}`: no clang on PATH");
        return;
    }
    let r = run(name, src);
    assert_eq!(r.code, Some(1), "`{name}` exit status");
    assert_eq!(r.stdout.trim(), expected.trim(), "`{name}` output");
    assert!(
        r.stderr.contains(stderr_has),
        "`{name}` stderr should mention {stderr_has:?}, got:\n{}",
        r.stderr
    );
    assert!(
        !r.stderr.contains("leak"),
        "`{name}` leaked on the error path:\n{}",
        r.stderr
    );
}

// ---------------------------------------------------------------- errors

const UNCAUGHT: &str = "\
class Res {
    name: string
}
fn load(n: int) -> Res {
    if n < 0 {
        throw Error(message: \"cannot load ${n}\", code: 12)
    }
    return Res(name: \"r${n}\")
}
print(load(1).name)
print(load(-1).name)
print(\"unreachable\")
";

#[test]
fn uncaught_error_stops_with_status_1_and_its_message() {
    check_fails(
        "uncaught",
        UNCAUGHT,
        "r1",
        "kode: error: cannot load -1 (code 12)",
    );
}

const PROPAGATE: &str = "\
fn level3(n: int) -> int {
    if n > 2 {
        throw \"too deep: ${n}\"
    }
    return n
}
fn level2(n: int) -> int {
    a = level3(n + 1)
    return a * 2
}
fn level1(n: int) -> int {
    return level2(n) + 100
}
print(level1(0))
print(level1(5) catch e {
    print(\"caught:\", e.message)
    -5
})
print(level1(5))
";

#[test]
fn errors_propagate_through_nested_calls() {
    check_fails(
        "propagate",
        PROPAGATE,
        "102\ncaught: too deep: 6\n-5",
        "kode: error: too deep: 6",
    );
}

const LOOP_ALLOC: &str = "\
class Item {
    label: string
    tags: [string] = []
}
fn risky(i: int) -> Item {
    it = Item(label: \"item ${i}\")
    it.tags.add(\"a\")
    it.tags.add(\"b\")
    if i % 7 == 3 {
        throw \"bad item ${i}\"
    }
    return it
}
total = 0
ok = 0
for i in 0..500 {
    r = risky(i) catch e {
        total += 1
        continue
    }
    ok += 1
    if len(r.tags) != 2 {
        print(\"broken\")
    }
}
print(ok, total)
i = 0
while i < 500 {
    v = risky(i * 3) catch e {
        i += 1
        continue
    }
    ok += 1
    i += 1
}
print(ok)
";

#[test]
fn errors_after_allocation_in_loops_do_not_leak() {
    check("loop_alloc", LOOP_ALLOC, "429 71\n857");
}

const NESTED_CATCH: &str = "\
fn inner(n: int) -> int {
    if n == 0 {
        throw \"inner zero\"
    }
    return n
}
sum = 0
for i in 0..4 {
    try {
        a = inner(i) catch e {
            sum += 1000
            throw \"outer ${e.message}\"
        }
        sum += a
    } catch outer {
        print(\"outer caught:\", outer.message)
        sum += 1
    }
}
print(sum)
fn early(n: int) -> string {
    x = inner(n) catch e {
        return \"returned from catch ${e.message}\"
    }
    return \"value ${x}\"
}
print(early(0), early(2))
";

#[test]
fn nested_catch_and_return_from_a_catch_block() {
    check(
        "nested_catch",
        NESTED_CATCH,
        "outer caught: outer inner zero\n1007\nreturned from catch inner zero value 2",
    );
}

// ----------------------------------------------------------- concurrency

const TASK_ERRORS: &str = "\
fn ok(n: int) -> int => n * 2
fn bad(n: int) -> int {
    if n % 2 == 0 {
        throw \"even ${n} rejected\"
    }
    return n
}
t1 = spawn ok(21)
t2 = spawn bad(4)
t3 = spawn bad(5)
print(await t1)
r = await t2 catch e {
    print(\"t2 failed:\", e.message)
    -1
}
print(r, await t3)
t4 = spawn bad(8)
try {
    print(await t4)
} catch e {
    print(\"t4 caught:\", e.message)
}
print(\"done\")
";

#[test]
fn task_errors_surface_at_await_and_are_catchable() {
    check(
        "task_errors",
        TASK_ERRORS,
        "42\nt2 failed: even 4 rejected\n-1 5\nt4 caught: even 8 rejected\ndone",
    );
}

const CHAN_REFS: &str = "\
class Msg {
    text: string
    nums: [int] = []
}
msgs: Channel<Msg> = channel()
m1 = Msg(text: \"first\")
m1.nums.add(1)
m1.nums.add(2)
send(msgs, m1)
send(msgs, Msg(text: \"second\"))
close(msgs)
a = recv(msgs)
b = recv(msgs)
print(a.text, a.nums.len(), b.text, b.nums.len())
c = recv(msgs) catch e {
    print(\"closed:\", e.message)
    Msg(text: \"fallback\")
}
print(c.text)
names: Channel<string> = channel()
send(names, \"alpha\")
send(names, \"beta\")
close(names)
print(recv(names), recv(names))
";

#[test]
fn channels_carry_objects_and_strings_and_drain_before_close() {
    check(
        "chan_refs",
        CHAN_REFS,
        "first 2 second 0\nclosed: receive from a closed channel\nfallback\nalpha beta",
    );
}

const CHAN_MPMC: &str = "\
work: Channel<int> = channel()
fn producer(ch: Channel<int>, base: int, n: int) {
    for i in 0..n {
        send(ch, base + i)
    }
}
fn consumer(in_ch: Channel<int>, n: int) -> int {
    total = 0
    for i in 0..n {
        total += recv(in_ch)
    }
    return total
}
p1 = spawn producer(work, 0, 10)
p2 = spawn producer(work, 100, 10)
c1 = spawn consumer(work, 10)
c2 = spawn consumer(work, 10)
await p1
await p2
r1 = await c1
r2 = await c2
close(work)
print(r1 + r2)
";

#[test]
fn two_producers_and_two_consumers() {
    check("chan_mpmc", CHAN_MPMC, "1090");
}

const TASK_MANY: &str = "\
fn make(n: int) -> string => \"task ${n}\"
tasks: [Task<string>] = []
for i in 0..300 {
    tasks.add(spawn make(i))
}
joined = 0
for t in tasks {
    s = await t
    if len(s) > 0 {
        joined += 1
    }
}
print(joined)
out: Channel<string> = channel()
producers: [Task<int>] = []
fn feed(ch: Channel<string>, n: int) -> int {
    for i in 0..5 {
        send(ch, \"m${n}-${i}\")
    }
    return 1
}
for i in 0..20 {
    producers.add(spawn feed(out, i))
}
got = 0
for i in 0..100 {
    s = recv(out)
    if len(s) > 0 {
        got += 1
    }
}
for p in producers {
    await p
}
close(out)
print(got)
";

#[test]
fn repeated_task_creation_and_many_strings_over_a_channel() {
    check("task_many", TASK_MANY, "300\n100");
}
