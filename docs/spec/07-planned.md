# 7. Planned (not implemented yet)

```
service Todos {                       # M8: services with budgets and effects
    memory 64mb
    get("/todos") uses db => todos
}
```

- M6 and M7 are done (chapters 12 and 13). Still missing from them: channels, `shared` classes, a database client.
- M8: resource budgets (`memory 64mb`), declared effects (`uses db`), typed `ask` / `tool` calls for AI.
- Later: `weak` references, `finally`, `int?`, optional narrowing, integer map keys, list sorting
  and `map`/`filter`, package dependencies and a registry, a language server.
