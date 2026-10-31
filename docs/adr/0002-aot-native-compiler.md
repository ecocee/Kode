# ADR 0002: Ahead-of-time native compiler
Status: accepted

Kode compiles to a native binary, with no VM and no interpreter. Reason: the main goal is lower
cloud cost, which needs low RAM, millisecond startup and one small file to deploy.
