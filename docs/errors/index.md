# Diagnostics reference

`kode explain <CODE>` prints this page for a specific code. Each code has a failing
test in `tests/compile-fail/`, which is the source of truth for the exact message.

## Lexer (E000x)

| Code | Meaning |
|---|---|
| E0001 | string is never closed |
| E0002 | unknown escape sequence |
| E0003 | `${` is never closed |
| E0004 | block comment is never closed |
| E0005 | integer literal is too large |
| E0006 | not a valid number |
| E0007 | unexpected character (including `;`) |

## Parser (E01xx)

| Code | Meaning |
|---|---|
| E0101 | expected the end of the line |
| E0102 | expected a value |
| E0103 | expected a specific token such as `)` or `{` |
| E0105 | function defined inside a block |
| E0106 | block is never closed |
| E0107 | expected `=` after a declared type |
| E0108 | a range loop has exactly one variable |
| E0110 | expected a name |
| E0118 | the expression, block or type nests deeper than 128 levels |

## Variables and expressions (E02xx)

| Code | Meaning |
|---|---|
| E0201 | unknown variable |
| E0202 | wrong type in assignment or declaration |
| E0203 | operator or function cannot be used with these types (also: a value with no text form, such as a class inside a list, used in `print` or `${}`) |
| E0204 | division by zero (constant) |
| E0205 | a call that returns nothing was used as a value |
| E0206 | unknown function |
| E0207 | wrong number of arguments |
| E0208 | wrong argument type |
| E0209 | condition is not a bool |
| E0210 | int and float mixed without conversion |
| E0211 | this value cannot be indexed or sliced |
| E0212 | wrong index type (lists need an int, maps need a string) |
| E0213 | list or map items have different types |
| E0214 | cannot loop over this value |
| E0215 | bad named argument |
| E0216 | missing argument |
| E0217 | unknown field or method |
| E0218 | this value has no fields |
| E0219 | optional type not supported here (an optional number only works as a local) |
| E0220 | value may be none and cannot be used directly |
| E0221 | `?.` or `??` used on something that is not an optional |
| E0222 | an empty list or map needs a declared type |
| E0223 | bad `match` value or pattern type |
| E0224 | `match` does not cover every case |
| E0225 | unknown enum variant |
| E0226 | `throw` needs a string or an `Error` |
| E0227 | `catch` block does not end with a value |
| E0228 | parameter has the same name as a field |
| E0230 | `spawn` needs one of your functions with int, float, bool, string or channel parameters |
| E0231 | `await` needs a task |
| E0232 | the `serve` handler must be a function `(string, string, string) -> string`, or a `Router` |
| E0233 | the route handler must be a function `(req: Request) -> Response` |
| E0240 | a channel is used before its payload type is known |
| E0241 | a closure parameter needs a type |
| E0242 | bad function value: `map`/`filter` callback, or `this` used inside a closure |
| E0250 | cannot infer a type parameter of a generic function |
| E0251 | a generic function cannot be used as a value |
| E0252 | a method cannot have type parameters yet |

## Declarations and statements (E03xx)

| Code | Meaning |
|---|---|
| E0301 | function, class, enum or method defined twice |
| E0302 | name is a built-in function |
| E0303 | unknown type name |
| E0304 | `=>` body does not match the declared type |
| E0305 | not every path returns a value |
| E0306 | parameter or field name used twice |
| E0307 | constant is not a plain value |
| E0308 | constant defined twice |

## Imports (E04xx)

| Code | Meaning |
|---|---|
| E0401 | imported file not found |
| E0402 | import cycle |
| E0403 | a module has top-level code |
| E0405 | the module does not define this name |
| E0406 | invalid `kode.toml` package manifest |
| E0407 | the package root folder does not exist |
| E0408 | a module alias is imported twice |
| E0409 | a module alias hides another declaration |
| E0410 | the module has no such member |
| E0411 | the member is private (not `pub`) |
| E0412 | a dependency is not a package (no `kode.toml`) |
| E0413 | dependency version mismatch, or a version without a path |
| E0414 | two packages share a name, or a package depends on itself |

## Warnings

| Code | Meaning |
|---|---|
| W0001 | variable assigned but never used (`--strict` turns it into an error) |
