# 12. Tasks (concurrency)

```
fn work(n: int) -> int {
    total = 0
    for i in 0..n { total += i }
    return total
}

t = spawn work(1000)            # starts a task on its own thread, gives a Task<int>
print(await t)                  # waits and gives the result

tasks: [Task<int>] = []
for i in 1..=4 { tasks.add(spawn work(i * 10)) }
sum = 0
for t in tasks { sum += await t }

r = await spawn fails() catch e { -1 }     # an error inside a task is raised by `await`
```

- `spawn f(args)` needs one of your functions. Its parameters can only be `int`, `float`, `bool`
  and `string` (E0230): tasks share nothing, so there are no data races. The result can be any type.
- `await` waits for the result (E0231 if the value is not a task). It can raise the task's error,
  so a function that uses `await` can fail like a function that uses `throw`.
- A task that is never awaited is waited for and freed when its handle goes away.
- Reference counts become atomic when the first task starts; programs without tasks pay nothing.
- `sleep(ms)` pauses, `now_ms()` reads a steady clock in milliseconds (differences only).
- Not yet: channels, shared classes, a thread pool (every task is a thread).
