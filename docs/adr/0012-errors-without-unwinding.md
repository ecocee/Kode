# ADR 0012: Errors use a pending-error slot, not stack unwinding
Status: accepted

`throw` stores the error and jumps; callers check a flag after calls that can fail.
It needs no unwinding tables, keeps the runtime tiny, costs nothing for code that cannot fail,
and lets reference counting release everything on the error path.
