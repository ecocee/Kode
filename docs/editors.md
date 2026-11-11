# Editors

Kode ships a language server. It is the compiler's own front end, so an editor shows
exactly what `kode check` reports: the same codes, the same hints, the same line numbers.

```
kode lsp        # JSON-RPC on stdin/stdout
```

What it does today:

| Feature | Notes |
|---|---|
| Diagnostics | pushed after every change, for the open buffer (unsaved changes included) |
| Hover | type of the expression under the cursor |
| Completion | keywords, top-level items, stdlib functions, namespaces, local variables; stdlib methods after `.` |
| Document symbols | functions, classes, methods, fields, enums, interfaces, constants, tests |
| Formatting | the same re-indenter as `kode fmt` |
| Go to definition | declaration of a call or name under the cursor |
| References | every use of a name in the document |
| Rename | renames a name through the whole document, refusing keywords |

Not yet: signature help, code actions, a completion that knows every standard-library
method's parameters.

## Visual Studio Code

There is no packaged extension yet; item 4.2 in `STATUS.md` tracks it as the
remaining work. A generic LSP client is the easiest route meanwhile: install
`vscode-languageclient` and register `kode` for the `source.kode` language.

Server settings, in `settings.json`:

```json
{
  "kode.server.path": "kode",
  "kode.trace.server": "off"
}
```

Starting the server from an extension:

```ts
import { LanguageClient } from "vscode-languageclient/node";

const client = new LanguageClient(
  "kode",
  "Kode",
  { command: "kode", args: ["lsp"] },
  { documentSelector: [{ scheme: "file", language: "kode" }] },
);
client.start();
```

If `kode` is not on `PATH`, point the command at the absolute path, for example
`/Users/you/Projects/Kode/target/debug/kode`.

## Neovim

Built-in LSP client, no plugin needed:

```lua
vim.filetype.add({ extension = { kode = "kode" } })

vim.api.nvim_create_autocmd("FileType", {
  pattern = "kode",
  callback = function(args)
    vim.lsp.start({
      name = "kode",
      cmd = { "kode", "lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({ "kode.toml" }, { upward = true })[1])
        or vim.fs.dirname(args.file),
    })
  end,
})
```

Useful mappings while the server is attached:

```lua
vim.keymap.set("n", "K", vim.lsp.buf.hover, { buffer = 0 })
vim.keymap.set("n", "gd", vim.lsp.buf.definition, { buffer = 0 })
vim.keymap.set("n", "<leader>rn", vim.lsp.buf.rename, { buffer = 0 })
```

## Helix

`languages.toml`:

```toml
[language-server.kode]
command = "kode"
args = ["lsp"]

[[language]]
name = "kode"
scope = "source.kode"
file-types = ["kode"]
comment-token = "//"
language-servers = ["kode"]
indent = { tab-width = 4, unit = "    " }
```

## Debugging the server

The server writes only protocol messages on stdout, so it is safe to pipe it. Anything
unexpected is reported on stderr:

```
kode lsp 2> /tmp/kode-lsp.log
```

A malformed message is logged and skipped; the server stays up.
