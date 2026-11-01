# 7. Planned (not implemented yet)

```
task = spawn fetch(1)                 # M6: concurrency
price = await task

serve(8080) {                         # M7: services
    get("/todos") => todos
}
```

- M6: `spawn` / `await`, channels, atomic reference counts, `shared` classes.
- M7: HTTP server, JSON, database client, string functions (`split`, `upper`, ...), file I/O.
- M8: resource budgets (`memory 64mb`), declared effects (`uses db`), typed `ask` / `tool` calls for AI.
- Later: `weak` references, interfaces, `finally`, `int?`, optional narrowing, integer map keys,
  list sorting and `map`/`filter`, qualified module names, a package manager, a language server.
