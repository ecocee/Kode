# 7. Planned (not implemented yet)

These are designed in the original syntax draft and arrive in the milestones of `docs/ROADMAP.md`.

```
class Account {                       # M3
    owner: string
    balance: int = 0
    fn deposit(amount: int) { balance += amount }
}

user: User? = none                    # M4
name = user?.name ?? "guest"
r = load("a.json") catch err { return }
throw "bad id"

task = spawn fetch(1)                 # M6
price = await task

serve(8080) {                         # M7
    get("/todos") => todos
}
```

Also planned: lists `[T]`, maps `{K: V}`, `for x in list`, enums, `match`, `import`, `test` blocks,
interfaces, `weak` references, resource budgets (`memory 64mb`), declared effects (`uses db`),
typed `ask` / `tool` calls for AI.
