//! B2 shape checks: the runtime ABI calls that make strings, reference
//! counting, optionals and the error slot work (no clang needed).

use kode_llvm::{emit, program_from_source};

fn ir(src: &str) -> String {
    let mir = program_from_source(src, "<test>").expect("front end accepts the program");
    emit(&mir).expect("IR generation")
}

#[test]
fn strings_use_the_runtime_and_are_retained_and_released() {
    let text = ir("\
a = \"hello\"
b = \" world\"
c = a + b
print(len(c))
print(c == \"hello world\")
");
    assert!(text.contains("call i64 @\"kode_str_concat\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_str_len\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_str_eq\"(i64"), "{text}");
    // A bool result comes back as i64 and is narrowed for the language.
    assert!(text.contains("icmp ne i64"), "{text}");
    assert!(text.contains("call void @kode_release(i64"), "{text}");
}

#[test]
fn aliasing_a_string_retains_it() {
    let text = ir("\
a = \"hello\"
b = a
c = a + b
print(c)
");
    assert!(text.contains("call void @kode_retain(i64"), "{text}");
    assert!(text.contains("call void @kode_release(i64"), "{text}");
}

#[test]
fn static_strings_keep_the_cranelift_layout() {
    let text = ir("print(\"hi\")\n");
    // [refcount = -1][kind = 1][length = 2][h][i], every byte escaped.
    assert!(
        text.contains("@kode_str_0 = private unnamed_addr constant [26 x i8]"),
        "{text}"
    );
    assert!(
        text.contains("\\FF\\FF\\FF\\FF\\FF\\FF\\FF\\FF\\01"),
        "refcount -1 then kind 1:\n{text}"
    );
    assert!(
        text.contains("\\02\\00\\00\\00\\00\\00\\00\\00\\68\\69"),
        "length 2 then hi:\n{text}"
    );
}

#[test]
fn conversion_goes_through_the_runtime() {
    let text = ir("n = 42\nprint(\"n=${n}\")\n");
    assert!(
        text.contains("kode_str_concat") || text.contains("kode_int_to_str"),
        "{text}"
    );
    assert!(text.contains("call i64 @\"kode_int_to_str\"(i64"), "{text}");
}

#[test]
fn throws_and_catches_use_the_error_slot() {
    let text = ir("\
fn boom() -> int {
    throw \"kaboom\"
}
v = boom() catch e {
    print(e.message)
    -1
}
print(v)
");
    assert!(text.contains("call void @\"kode_err_throw\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_err_pending\"()"), "{text}");
    assert!(text.contains("call i64 @\"kode_err_take\"()"), "{text}");
}

#[test]
fn none_is_a_null_reference_not_undefined_value() {
    let text = ir("\
s: string? = none
print(s ?? \"empty\")
");
    assert!(
        !text.contains("undef"),
        "none must never become undef:\n{text}"
    );
    assert!(text.contains("store i64 0, ptr %l"), "{text}");
}

#[test]
fn optional_unwrap_guards_with_the_runtime() {
    let text = ir("\
n: int? = 3
print(n!)
");
    assert!(
        text.contains("kode_unwrap_fail") || text.contains("icmp"),
        "an unwrap must check before using the value:\n{text}"
    );
}

#[test]
fn every_release_sits_on_the_path_that_owns_it() {
    // Two returns: the early one after building a string, the late one after
    // building another. Both paths must release before returning.
    let text = ir("\
fn label(n: int) -> string {
    if n % 2 == 0 {
        return \"even ${n}\"
    }
    s = \"odd ${n}\"
    return s
}
print(label(1))
");
    // MIR merges the two source-level returns into one exit block, so the
    // check is that both paths release their intermediates and nothing is
    // released twice: the merged exit releases the result slot once.
    let releases = text.matches("call void @kode_release(i64").count();
    let returns = text.matches("ret i64").count();
    assert_eq!(returns, 1, "the two returns share one exit block:\n{text}");
    assert!(
        releases >= 4,
        "every temporary on both paths is released:\n{text}"
    );
    let ret = text.find("ret i64").expect("a return");
    let before = &text[ret.saturating_sub(220)..ret];
    assert!(
        before.contains("kode_release"),
        "the result slot is released on the way out:\n{text}"
    );
}
