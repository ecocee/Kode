pub const HELP: &str = "\
kode - the Kode compiler

USAGE:
    kode run <file.kode> [-- args]    compile and run a program (args go to the program)
    kode build <file.kode> [-o out]   compile to a native binary
    kode check <file.kode>            check for errors only (fast)
    kode test [file-or-folder]        run the `test` blocks (default: current folder)
    kode fmt <file-or-folder>...      format source files (`--check` only reports)
    kode explain <CODE>               explain an error code, for example E0201
    kode tokens <file.kode>           print the tokens (debugging)
    kode ast <file.kode>              print the syntax tree (debugging)
    kode mir <file.kode>              print the MIR (debugging)
    kode lsp                          language server on stdio (for editors)
    kode doctor                       check the compiler, linker and package setup
    kode doc <folder> [-o dir]        write an HTML reference for `pub` items
    kode init [name]                  make the current folder a package
    kode new <name>                   make a package in a new folder
    kode add <name> --path <folder>   add a path dependency
    kode remove <name>                remove a dependency
    kode install                      resolve dependencies, write kode.lock

OPTIONS:
    --format=json     print diagnostics as JSON (for editors and AI agents)
    --strict          treat warnings as errors
    --release         strip symbols from the built binary
    --debug           do not optimize the generated code (faster to compile)
    --backend=llvm    accepted for compatibility; LLVM is the only backend
    --check           with `fmt`: do not write, exit 1 if a file would change
                      with `install`: do not write kode.lock, exit 1 if it is stale
    --path <folder>   with `add`: the dependency's folder
    -o <file>         output file for `build`
    -h, --help        show this help
    -V, --version     show the version
";

#[derive(Default)]
pub struct Options {
    pub command: String,
    pub files: Vec<String>,
    pub output: Option<String>,
    pub json: bool,
    pub strict: bool,
    pub release: bool,
    pub debug: bool,
    pub check: bool,
    pub simple: bool,
    pub path: Option<String>,
    pub program_args: Vec<String>,
}

impl Options {
    pub fn file(&self) -> Option<&str> {
        self.files.first().map(|s| s.as_str())
    }
}

pub fn parse() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut opts = Options::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => opts.command = "help".into(),
            "-V" | "--version" => opts.command = "version".into(),
            "--format=json" => opts.json = true,
            "--strict" => opts.strict = true,
            "--release" => opts.release = true,
            "--debug" => opts.debug = true,
            "--simple" => opts.simple = true,
            "--" => {
                opts.program_args = args.by_ref().collect();
                break;
            }
            "--check" => opts.check = true,
            "--stdio" => {}
            "-o" => opts.output = Some(args.next().ok_or("`-o` needs a file name")?),
            "--path" => opts.path = Some(args.next().ok_or("`--path` needs a folder")?),
            other if other.starts_with("--backend=") => {
                let name = &other["--backend=".len()..];
                match name {
                    "llvm" => {}
                    "cranelift" => {
                        return Err(
                            "the Cranelift backend was removed; LLVM is the only backend".into()
                        )
                    }
                    _ => return Err(format!("unknown backend `{name}`; LLVM is the only backend")),
                }
            }
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            other => {
                if opts.command.is_empty() {
                    opts.command = other.to_string();
                } else {
                    opts.files.push(other.to_string());
                }
            }
        }
    }
    Ok(opts)
}
