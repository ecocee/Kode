# ADR 0008: Variables are block scoped
Status: accepted (changes the first syntax draft, which used function scope)

A variable first assigned inside `{ }` ends with that block. This removes a class of bugs
where a variable is used after a branch that did not set it.
