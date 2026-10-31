# ADR 0005: Automatic reference counting
Status: accepted for strings, proposed for classes

No garbage collector (pauses, extra RAM) and no ownership annotations (hard to learn).
The compiler inserts retain and release in MIR. Cycles need `weak` references (planned).
