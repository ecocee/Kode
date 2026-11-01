# How errors work

`throw` does not unwind the stack. It stores the error object in one global slot of the runtime
and jumps to the nearest handler in the same function, or out of the function.

1. The checker works out which functions can fail: a function throws directly (`throw`,
   `assert`) or calls one that does. Calls to functions that cannot fail cost nothing extra
   (recursive Fibonacci runs at full speed).
2. After a call to a function that can fail, the compiled code calls `kode_err_pending`.
   If an error is set it jumps to the `catch` block of the innermost `try` or `catch`, or to
   the function's error exit.
3. The error exit releases the function's variables and returns a default value, so the caller
   sees the pending error too.
4. `catch` takes the error out of the slot (`kode_err_take`), so it belongs to the catch variable.
5. At the top of the program, an error that is still pending prints `kode: error: message`
   and exits with code 1.

Errors are ordinary `Error` objects (a message and a code), so they are reference counted
and freed like everything else.
