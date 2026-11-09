//! B4 shape checks: object allocation, field layout, methods and dispatch.
//!
//! The layout asserted here is the runtime's: `[rc][kind][refmask]` then one
//! 8-byte slot per field from offset 24. `kode_obj_new_class(nfields, refmask,
//! class_id)` allocates `24 + 8 * nfields` bytes.

use kode_llvm::{emit, program_from_source};

fn ir(src: &str) -> String {
    let mir = program_from_source(src, "<test>").expect("front end accepts the program");
    emit(&mir).expect("IR generation")
}

#[test]
fn allocation_passes_the_field_count_and_class_id() {
    let text = ir("\
class Account {
    owner: string
    balance: int = 0
    rate: float = 1.5
}
a = Account(owner: \"Asha\")
print(a.balance)
");
    assert!(
        text.contains("call i64 @\"kode_obj_new_class\"(i64 3, i64 1, i64"),
        "three fields, one reference slot:\n{text}"
    );
}

#[test]
fn fields_are_slots_from_offset_24() {
    let text = ir("\
class Holder {
    inner: int
    items: [int] = []
}
h = Holder(inner: 1)
print(h.inner, h.items.len())
");
    assert!(text.contains("i64 24"), "first field slot:\n{text}");
    assert!(text.contains("i64 32"), "second field slot:\n{text}");
    assert!(text.contains("getelementptr i8, ptr"), "{text}");
    assert!(text.contains("load i64, ptr"), "{text}");
}

#[test]
fn assigning_a_string_field_releases_the_old_value() {
    let text = ir("\
class Box { v: string = \"x\" }
b = Box()
b.v = \"y\"
");
    // A string literal has a static refcount, so it is not retained; the value
    // it replaces is still released. MIR stores the new value into the field,
    // then releases the value that was
    // there, then releases the object at the end of scope. Match that order.
    let field = text.rfind("i64 24").expect("field slot");
    let tail = &text[field..];
    assert!(
        tail.contains("call void @kode_release(i64"),
        "the previous field value is released after the store:\n{text}"
    );
    assert!(
        text.matches("call void @kode_release(i64").count() >= 2,
        "the old field value and the object are both released:\n{text}"
    );
}

#[test]
fn methods_are_plain_calls_with_the_receiver_first() {
    let text = ir("\
class Account {
    owner: string
    fn summary() -> string => owner
}
a = Account(owner: \"Asha\")
print(a.summary())
");
    assert!(
        text.contains("define i64 @\"kode_fn_Account.summary\"(i64"),
        "{text}"
    );
    assert!(
        text.contains("call i64 @\"kode_fn_Account.summary\"(i64"),
        "{text}"
    );
}

#[test]
fn interfaces_get_a_class_id_dispatch_function() {
    let text = ir("\
interface Shape {
    fn area() -> int
}
class Square implements Shape {
    side: int
    fn area() -> int => side * side
}
c: Shape = Square(3)
print(c.area())
");
    assert!(
        text.contains("define i64 @\"kode_fn_Shape.area\"(i64"),
        "the generated dispatch function:\n{text}"
    );
    assert!(text.contains("call i64 @\"kode_fn_Square.area\""), "{text}");
}

#[test]
fn optional_objects_are_null_and_unwrap_checks() {
    let text = ir("\
class P {
    x: int = 1
}
p: P? = none
print(p!.x)
");
    assert!(text.contains("store i64 0, ptr %l"), "{text}");
    assert!(text.contains("call void @\"kode_unwrap_fail\"(i64"), "{text}");
}

#[test]
fn generic_instantiations_are_named_per_type() {
    let text = ir("\
class Pair {
    a: int
    b: int
}
fn first<T>(xs: [T]) -> T {
    return xs[0]
}
xs = [Pair(a: 1, b: 2)]
print(first(xs).a)
");
    assert!(
        text.contains("define i64 @\"kode_fn_first.cls_Pair\""),
        "{text}"
    );
}

#[test]
fn objects_never_use_undef() {
    let text = ir("\
class Box {
    v: string = \"x\"
    n: int = 0
}
b = Box()
b.v = b.v + \"!\"
print(b.v, b.n)
");
    assert!(!text.contains("undef"), "no undefined object state:\n{text}");
}
