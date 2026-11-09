//! Shape checks on the generated IR (no clang needed).

use kode_llvm::{emit, program_from_source};

fn ir(src: &str) -> String {
    let mir = program_from_source(src, "<test>").expect("front end accepts the program");
    emit(&mir).expect("IR generation")
}

#[test]
fn functions_arguments_and_calls() {
    let text = ir("\
fn add(a: int, b: int) -> int {
    return a + b
}
print(add(2, 3))
");
    assert!(
        text.contains("define i64 @\"kode_fn_add\"(i64 %p0, i64 %p1)"),
        "{text}"
    );
    assert!(
        text.contains("call i64 @\"kode_fn_add\"(i64 2, i64 3)"),
        "{text}"
    );
    assert!(text.contains("call void @\"kode_print_int\""), "{text}");
    assert!(text.contains("define i32 @main()"), "{text}");
    assert!(text.contains("call void @kode_rt_init()"), "{text}");
}

#[test]
fn comparisons_branches_and_loops() {
    let text = ir("\
total = 0
for i in 0..10 {
    total += i
}
print(total)
");
    assert!(text.contains("icmp slt i64"), "loop condition:\n{text}");
    assert!(text.contains("br i1"), "conditional branch:\n{text}");
    assert!(text.contains("label %bb"), "{text}");
}

#[test]
fn division_calls_the_checked_helper() {
    let text = ir("print(7 / 2)\n");
    assert!(text.contains("call void @kode_div_zero()"), "{text}");
    assert!(text.contains("sdiv i64"), "{text}");
    assert!(text.contains("icmp eq i64 2, -1"), "{text}");
}

#[test]
fn shifts_guard_the_count() {
    let text = ir("print(1 << 64)\n");
    assert!(
        text.contains("add i64 0, 0") || text.contains("shl i64"),
        "{text}"
    );
    let text = ir("n = 64\nprint(1 << n)\n");
    assert!(text.contains("icmp uge i64"), "{text}");
    assert!(text.contains("select i1"), "{text}");
}

#[test]
fn floats_are_double() {
    let text = ir("x = 1.5 * 2.0\nprint(x)\n");
    assert!(text.contains("fmul double"), "{text}");
    assert!(
        text.contains("call void @\"kode_print_float\"(double"),
        "{text}"
    );
}

#[test]
fn strings_use_the_static_layout() {
    let text = ir("print(\"hi\")\n");
    assert!(
        text.contains("@kode_str_0 = private unnamed_addr constant"),
        "{text}"
    );
    assert!(text.contains("ptrtoint ptr @kode_str_0 to i64"), "{text}");
}
