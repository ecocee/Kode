# 1. Lexical structure

- Source files end in `.kode` and are UTF-8.
- A statement ends at the end of the line. There are no semicolons (`;` is an error, E0007).
- A line continues when it ends with a binary operator or a comma, or when it is inside `( )`.
- Comments: `// to end of line` and `/* block */`.
- Identifiers: letters, digits and `_`, not starting with a digit.
- Keywords: `fn const if else while for in break continue return true false and or not class enum match import from pub throw try catch none test`.
- Integers: `42`, `1_000`. Also base 2, 8 and 16: `0b1010`, `0o755`, `0x6b` (prefix
  in either case, `_` between digits). A literal that does not fit an `int` is E0006.
- Floats: `3.14`, `2.5e-3`, `1e6`. A range `1..5` is not a float, and `0x` never
  starts one.
- Strings: `"text"` on one line. Escapes: `\n \t \r \0 \" \\ \$`.
  Interpolation: `"${expression}"`; any expression may go inside, including nested strings.
