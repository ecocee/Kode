//! `kode explain E0201`

const INDEX: &str = include_str!("../../../docs/errors/index.md");

/// (code, what goes wrong, example that fails, how it looks fixed)
const DETAILS: &[(&str, &str, &str, &str)] = &[
    (
        "E0118",
        "The syntax tree is too deep for the compiler to walk. Recursive passes (parser, type checker, MIR lowering, code generator) each need a stack frame per level, so the limit is 128. Long `+` chains count one level per term.",
        "x: int = 1 + 1 + 1 + ... (129 terms) — or 129 nested parentheses, or 43 nested if blocks",
        "x: int = a + b\ny: int = x + c\n// or build the long list with a loop",
    ),
    ("E0201", "A name is used before it was created, or it is spelled differently.", "name = \"Asha\"\nprint(nme)", "name = \"Asha\"\nprint(name)"),
    ("E0202", "A variable keeps the type of its first value. A different type cannot be stored in it.", "x = 1\nx = \"one\"", "x = 1\nx = 2"),
    (
        "E0203",
        "An operator or function was used with types it does not support. A list or map also needs printable items: a class has fields, not one text form.",
        "class U {\n    n: int\n}\nprint([U(n: 1)])",
        "print([1, 2])\n// for a list of classes, loop and print a field",
    ),
    ("E0206", "A function that does not exist was called.", "prnt(\"hi\")", "print(\"hi\")"),
    ("E0207", "A function was called with the wrong number of arguments.", "fn f(a: int) -> int => a\nprint(f(1, 2))", "print(f(1))"),
    ("E0208", "An argument has the wrong type.", "fn f(a: int) -> int => a\nprint(f(\"x\"))", "print(f(1))"),
    ("E0209", "A condition must be a bool. Kode has no truthy or falsy values.", "x = 1\nif x {\n}", "if x != 0 {\n}"),
    ("E0210", "int and float never mix silently.", "print(1 + 2.5)", "print(float(1) + 2.5)"),
    ("E0213", "All items of a list must have the same type.", "xs = [1, \"a\"]", "xs = [1, 2]"),
    ("E0216", "A class value needs every field that has no default.", "class A {\n    x: int\n}\na = A()", "a = A(x: 1)"),
    ("E0217", "A class has no field or method with this name.", "a.y", "a.x"),
    (
        "E0219",
        "An optional number works as a local variable and as a class field, but not yet as a parameter, a return type, or inside a list or map.",
        "fn f(x: int?) -> int {\n    return 0\n}",
        "fn f(x: int, present: bool) -> int {\n    return 0\n}",
    ),
    ("E0220", "A value that may be none cannot be used directly.", "fn f(a: A?) -> int {\n    return a.x\n}", "return a!.x\n// or: a?.name ?? \"?\""),
    ("E0222", "An empty list or map has no type until you declare one.", "xs = []", "xs: [int] = []"),
    ("E0224", "A `match` must handle every case. Add the missing cases or a `_ =>` arm.", "match x {\n    1 => print(1)\n}", "match x {\n    1 => print(1)\n    _ => print(0)\n}"),
    ("E0227", "A `catch` block must end with a value (when used as an expression) or leave with return, break, continue or throw.", "v = f() catch e {\n    print(e.message)\n}", "v = f() catch e {\n    print(e.message)\n    0\n}"),
    (
        "E0417",
        "An enum variant with fields is a constructor: it is built with its fields, not used as a value.",
        "enum Shape {\n    Circle(radius: float)\n}\ns = Shape.Circle",
        "s = Shape.Circle(radius: 1.0)",
    ),
    (
        "E0419",
        "A payload enum may hold another variant than the fields you asked for, so read them after `is` narrowed the value.",
        "print(s.radius)",
        "if s is Shape.Circle {\n    print(s.radius)\n}",
    ),
    (
        "E0420",
        "`==` on a payload enum would compare two heap addresses. Compare the variant with `match`.",
        "print(a == b)",
        "match a {\n    Shape.Circle => print(\"circle\")\n    Shape.Empty => print(\"empty\")\n}",
    ),
    (
        "E0303",
        "A type name is unknown. Kode has int (64-bit), float (64-bit), bool, string, classes, interfaces and enums, lists [T], maps {string: T}, optionals T?, Task<T>, Channel<T> and function types fn(T) -> U. Fixed-width numbers such as u8 or i32 are planned, not available.",
        "x: u8 = 1",
        "x: int = 1",
    ),
    ("E0301", "A function, class or enum name is used twice (also across imported files).", "fn f() {\n}\nfn f() {\n}", "fn f() {\n}\nfn g() {\n}"),
    ("E0305", "A function that returns a value must return on every path.", "fn f(a: int) -> int {\n    if a > 0 {\n        return 1\n    }\n}", "fn f(a: int) -> int {\n    if a > 0 {\n        return 1\n    }\n    return 0\n}"),
    ("E0401", "An imported file was not found. The path is relative to the importing file; `.kode` is added for you.", "import \"./missing\"", "import \"./util\""),
    ("W0001", "A variable is assigned but never read. This often means a typo in a name.", "totl = 5\nprint(\"hi\")", "total = 5\nprint(total)"),
];

fn title(code: &str) -> Option<String> {
    for line in INDEX.lines() {
        let cells: Vec<&str> = line.split('|').map(|c| c.trim()).collect();
        if cells.len() >= 3 && cells[1] == code {
            return Some(cells[2].to_string());
        }
    }
    None
}

pub fn explain(code: &str) -> Option<String> {
    let code = code.to_uppercase();
    let t = title(&code)?;
    let mut out = format!("{code}: {t}\n");
    if let Some((_, why, bad, good)) = DETAILS.iter().find(|d| d.0 == code) {
        out.push_str(&format!(
            "\n{why}\n\nExample that fails:\n\n{}\n\nFixed:\n\n{}\n",
            indent(bad),
            indent(good)
        ));
    }
    Some(out)
}

fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
