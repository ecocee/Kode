# ADR 0006: Classes without inheritance
Status: proposed (applies from M3)

Use composition and interfaces. Deep hierarchies are hard to read, hard for AI agents, and
complicate the compiler. Inheritance can be added later; removing it later would be impossible.
