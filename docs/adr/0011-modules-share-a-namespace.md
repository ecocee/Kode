# ADR 0011: Imported files are merged into one namespace
Status: accepted for v0.2

`import` loads the file, and its functions, constants, classes and enums join the program.
Name clashes are errors. This is small, simple, and makes every message point at the right file.
`pub` is parsed but not enforced yet. Qualified names (`util.add`) and real visibility are planned.
