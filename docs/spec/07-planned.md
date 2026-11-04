# 7. Planned (not implemented yet)

```
service Todos {                       # M8: services with budgets and effects
    memory 64mb
    get("/todos") uses db => todos
}
```

- M6 and M7 are done (chapters 12 and 13). Still missing from them: channels, `shared` classes, JSON, an HTTP client, a database client.
- M8: resource budgets (`memory 64mb`), declared effects (`uses db`), typed `ask` / `tool` calls for AI.
- Later: `weak` references, interfaces, `finally`, `int?`, optional narrowing, integer map keys,
  list sorting and `map`/`filter`, qualified module names, a package manager, a language server.
