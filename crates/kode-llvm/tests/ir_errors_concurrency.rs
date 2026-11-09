//! B5 shape checks: the error protocol and the task/channel ABI.
//!
//! Errors are not exceptions in the IR: `throw` stores the value through
//! `kode_err_throw`, callers poll `kode_err_pending`, and a `catch` takes it
//! with `kode_err_take`. Tasks take a thunk address as an `i64` plus a "returns
//! a reference" flag; channels take the same flag on send, mirroring the
//! runtime signatures in `kode-rt`.

use kode_llvm::{emit, program_from_source};

fn ir(src: &str) -> String {
    let mir = program_from_source(src, "<test>").expect("front end accepts the program");
    emit(&mir).expect("IR generation")
}

#[test]
fn throw_sets_the_pending_error_instead_of_unwinding() {
    let text = ir("\
fn bad() -> int {
    throw \"nope\"
}
bad()
");
    assert!(text.contains("call void @\"kode_err_throw\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_err_pending\"()"), "{text}");
    assert!(!text.contains("invoke"), "no LLVM unwinding: {text}");
    assert!(!text.contains("landingpad"), "no landing pads: {text}");
}

#[test]
fn catch_takes_the_pending_error() {
    let text = ir("\
fn bad() -> int {
    throw \"nope\"
}
v = bad() catch e {
    print(e.message)
    -1
}
print(v)
");
    assert!(text.contains("call i64 @\"kode_err_take\"()"), "{text}");
}

#[test]
fn spawn_passes_a_thunk_and_the_reference_flag() {
    let text = ir("\
fn greet(n: int) -> string => \"hi ${n}\"
t = spawn greet(1)
print(await t)
");
    assert!(
        text.contains(
            "call i64 @\"kode_task_spawn\"(i64 ptrtoint (ptr @kode_thunk_0 to i64), i64 %t14, i64 1)"
        ),
        "a string-returning task flags its result as a reference:\n{text}"
    );
    assert!(text.contains("define i64 @\"kode_thunk_0\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_task_await\"(i64"), "{text}");
}

#[test]
fn channels_use_the_runtime_channel_calls() {
    let text = ir("\
class Msg {
    text: string
}
ch: Channel<Msg> = channel()
send(ch, Msg(text: \"a\"))
m = recv(ch)
close(ch)
print(m.text)
");
    assert!(text.contains("call i64 @\"kode_chan_new\"()"), "{text}");
    assert!(
        text.contains("call void @\"kode_chan_send\"(i64 %t") && text.contains(", i64 1)"),
        "a class message is sent as a reference:\n{text}"
    );
    assert!(text.contains("call i64 @\"kode_chan_recv\"(i64"), "{text}");
    assert!(text.contains("call void @\"kode_chan_close\"(i64"), "{text}");
    assert!(text.contains("define") && !text.contains("undef"), "{text}");
}

#[test]
fn a_task_list_keeps_tasks_as_ordinary_references() {
    let text = ir("\
fn work(n: int) -> int => n
tasks: [Task<int>] = []
for i in 0..2 {
    tasks.add(spawn work(i))
}
for t in tasks {
    print(await t)
}
");
    assert!(text.contains("call i64 @\"kode_task_spawn\"(i64"), "{text}");
    // The list is created by the runtime; `add` takes the B3 inline path.
    assert!(text.contains("call i64 @\"kode_list_new\"(i64 1)"), "{text}");
}

#[test]
fn errors_from_a_catch_block_are_still_pending_values() {
    let text = ir("\
fn inner(n: int) -> int {
    if n == 0 {
        throw \"inner\"
    }
    return n
}
v = inner(0) catch e {
    throw \"outer ${e.message}\"
}
print(v)
");
    assert!(
        text.matches("call void @\"kode_err_throw\"(i64").count() >= 2,
        "both throws go through the same call:\n{text}"
    );
}
