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
- Channels are implemented (see below); not yet: shared classes and a thread pool
  (every task is a thread).

## Channels

```
jobs: Channel<int> = channel()
done: Channel<string> = channel()

fn produce(out: Channel<int>, last: Channel<string>) {
    for i in 0..5 {
        send(out, i * 2)
    }
    send(last, "finished")
}

t = spawn produce(jobs, done)

sum = 0
for i in 0..5 {
    sum += recv(jobs)      # waits until a value is there
}
print(recv(done))
await t
close(jobs)
```

- `Channel<T>` is written like `Task<T>`. `channel()` has no payload of its own:
  the declared type gives it one (E0240), the same way `[int]` types an empty list.
- `send(ch, v)` queues the value (it does not block), `recv(ch)` waits, and
  `close(ch)` is idempotent. Sending on a closed channel, or receiving from one
  that is closed and empty, raises a catchable error.
- Heap payloads are retained when sent and released when received or when the
  channel is dropped, so a channel may outlive its sender.
- A task may take channels as parameters; that is how two tasks share state. It
  still may not take a class, list or map.
- Not yet: bounded channels (`channel(16)`), shared classes, a thread pool
  (every task is a thread).
