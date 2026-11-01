# Error codes

Every message has a stable code. `kode check --format=json` prints them as JSON, and
`kode explain E0201` shows a longer explanation with an example. Each code has a test in
`tests/compile-fail/`.

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
| E0111 | missing or invalid type |
| E0112 | this cannot be assigned to |
| E0114 | expected a field or a method in a class |
| E0115 | map keys must be strings |
| E0116 | expected a quoted file path or name |

## Type checker (E02xx)

| Code | Meaning |
|---|---|
| E0201 | unknown variable |
| E0202 | wrong type in assignment or declaration |
| E0203 | operator or function cannot be used with these types |
| E0204 | division by zero (constant) |
| E0205 | a call that returns nothing was used as a value |
| E0206 | unknown function |
| E0207 | wrong number of arguments |
| E0208 | wrong argument type |
| E0209 | condition is not a bool |
| E0210 | int and float mixed without conversion |
| E0211 | this value cannot be indexed |
| E0212 | wrong index type (lists need an int, maps need a string) |
| E0213 | list or map items have different types |
| E0214 | cannot loop over this value |
| E0215 | bad named argument |
| E0216 | missing argument |
| E0217 | unknown field or method |
| E0218 | this value has no fields |
| E0219 | optional type or `?.` not supported for this type |
| E0220 | value may be none and cannot be used directly |
| E0221 | `?.` or `??` used on something that is not an optional |
| E0222 | an empty list or map needs a declared type |
| E0223 | bad `match` value or pattern type |
| E0224 | `match` does not cover every case |
| E0225 | unknown enum variant |
| E0226 | `throw` needs a string or an `Error` |
| E0227 | `catch` block does not end with a value |
| E0228 | parameter has the same name as a field |

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
| E0310 | `return` value has the wrong type |
| E0311 | `return` outside a function |
| E0312 | assignment to a constant |
| E0313 | variable already declared |
| E0314 | `break` or `continue` outside a loop |

## Modules (E04xx)

| Code | Meaning |
|---|---|
| E0401 | imported file not found |
| E0402 | import cycle |
| E0403 | a module has top-level code |
| E0405 | the module does not define this name |

## Warnings

| Code | Meaning |
|---|---|
| W0001 | variable assigned but never used (`--strict` turns it into an error) |
