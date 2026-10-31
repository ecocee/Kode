# ADR 0009: No implicit conversions between numbers
Status: accepted

`int` and `float` never mix silently (E0210). Use `int(x)` and `float(x)`.
`int / int` is integer division. `+ - *` wrap on overflow for now; checked arithmetic is planned.
