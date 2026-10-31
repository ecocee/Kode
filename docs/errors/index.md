# Error codes

Every message has a stable code. `kode check --format=json` prints them as JSON.
Each code has a test in `tests/compile-fail/`.

## Lexer (E000x)

| Code | Meaning |
|---|---|
| E0001 | string is never closed |
| E0002 | unknown escape sequence |
| E0003 | `${` is never closed |
| E0004 | block comment is never closed |
| E0005 | integer literal is too large |
| E0006 | not a valid number |
| E0007 | unexpected character (including `;`, `!`, `.`) |

## Parser (E01xx)

| Code | Meaning |
|---|---|
| E0101 | expected the end of the line |
| E0102 | expected a value |
| E0103 | expected a specific token such as `)` or `{` |
| E0105 | function defined inside a block |
| E0106 | block is never closed |
| E0107 | expected `=` after a declared type |
| E0108 | `for` needs a loop variable name |
| E0109 | `for` needs a range such as `0..10` |
| E0110 | expected a name |
| E0111 | missing or invalid type |

## Type checker (E02xx)

| Code | Meaning |
|---|---|
| E0201 | unknown variable |
| E0202 | wrong type in assignment or declaration |
| E0203 | operator cannot be used with these types |
| E0204 | division by zero (constant) |
| E0205 | a call that returns nothing was used as a value |
| E0206 | unknown function |
| E0207 | wrong number of arguments |
| E0208 | wrong argument type |
| E0209 | condition is not a bool |
| E0210 | int and float mixed without conversion |

## Declarations and statements (E03xx)

| Code | Meaning |
|---|---|
| E0301 | function defined twice |
| E0302 | name is a built-in function |
| E0303 | unknown type name |
| E0304 | `=>` body does not match the declared type |
| E0305 | not every path returns a value |
| E0306 | parameter name used twice |
| E0307 | constant is not a plain value |
| E0308 | constant defined twice |
| E0310 | `return` value has the wrong type |
| E0311 | `return` outside a function |
| E0312 | assignment to a constant |
| E0313 | variable already declared |
| E0314 | `break` or `continue` outside a loop |

## Warnings

| Code | Meaning |
|---|---|
| W0001 | variable assigned but never used (`--strict` turns it into an error) |
