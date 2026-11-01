# ADR 0013: Optionals only for heap types in v0.2
Status: accepted (limitation)

`none` is a null reference, so `string?`, `[T]?`, `{string: T}?` and `Class?` cost nothing.
`int?` would need boxing; it is planned. Narrowing after `x != none` is planned; until then use `!` and `??`.
