//! B3 shape checks: collection operations that the reference lowering inlines must be
//! inlined here too, with the same bounds check and the same slow paths.

use kode_llvm::{emit, program_from_source};

fn ir(src: &str) -> String {
    let mir = program_from_source(src, "<test>").expect("front end accepts the program");
    emit(&mir).expect("IR generation")
}

#[test]
fn length_and_item_reads_are_inlined() {
    let text = ir("\
xs = [1, 2, 3]
print(xs.len())
print(xs[1])
");
    assert!(
        !text.contains("call i64 @\"kode_list_len\""),
        "len must load the header:\n{text}"
    );
    assert!(
        !text.contains("call i64 @\"kode_list_get\""),
        "an item read must not call the runtime:\n{text}"
    );
    assert!(text.contains("getelementptr i8, ptr"), "{text}");
    assert!(text.contains("i64 16"), "length field:\n{text}");
    assert!(text.contains("call void @\"kode_list_oob\"(i64"), "{text}");
    assert!(
        text.contains("icmp uge i64"),
        "unsigned bounds check:\n{text}"
    );
}

#[test]
fn packed_reads_load_one_byte() {
    let text = ir("\
flags: [bool] = [true, false]
print(flags[1])
");
    assert!(text.contains("load i8, ptr"), "{text}");
    assert!(text.contains("zext i8"), "{text}");
}

#[test]
fn push_has_a_fast_path_and_a_runtime_slow_path() {
    let text = ir("\
xs: [int] = []
xs.add(1)
");
    assert!(text.contains(".fast:"), "{text}");
    assert!(text.contains(".slow:"), "{text}");
    assert!(text.contains(".join:"), "{text}");
    assert!(text.contains("call void @\"kode_list_push\"(i64"), "{text}");
    assert!(text.contains("load i64, ptr"), "{text}");
    assert!(text.contains("add i64"), "new length:\n{text}");
}

#[test]
fn packed_push_stores_a_byte() {
    let text = ir("\
flags: [bool] = []
flags.add(true)
");
    assert!(
        text.contains("call void @\"kode_list_push8\"(i64"),
        "{text}"
    );
    assert!(text.contains("store i8"), "{text}");
}

#[test]
fn setting_a_reference_item_stays_in_the_runtime() {
    let text = ir("\
names: [string] = []
names.add(\"a\")
names[0] = \"b\"
");
    assert!(
        text.contains("call void @\"kode_list_set\"(i64"),
        "a ref item must release the old value in the runtime:\n{text}"
    );
}

#[test]
fn slices_use_the_runtime() {
    let text = ir("\
xs = [1, 2, 3, 4]
print(xs[1..3])
");
    assert!(text.contains("call i64 @\"kode_list_slice\"(i64"), "{text}");
}

#[test]
fn maps_and_json_use_the_runtime() {
    let text = ir("\
ages: {string: int} = {}
ages[\"a\"] = 1
print(ages.has(\"a\"), ages[\"a\"])
doc = json_parse(\"{\\\"k\\\":1}\")
print(doc[\"k\"].as_int())
");
    assert!(text.contains("call i64 @\"kode_map_has\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_map_get\"(i64"), "{text}");
    assert!(text.contains("call i64 @\"kode_json_parse\"(i64"), "{text}");
}
