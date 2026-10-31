pub const HELP: &str = "\
kode - the Kode compiler

USAGE:
    kode run <file.kode>              compile and run a program
    kode build <file.kode> [-o out]   compile to a native binary
    kode check <file.kode>            check for errors only (fast)
    kode tokens <file.kode>           print the tokens (debugging)
    kode ast <file.kode>              print the syntax tree (debugging)
    kode mir <file.kode>              print the MIR (debugging)

OPTIONS:
    --format=json     print diagnostics as JSON (for editors and AI agents)
    --strict          treat warnings as errors
    --release         optimize the generated code
    -o <file>         output file for `build`
    -h, --help        show this help
    -V, --version     show the version
";

#[derive(Default)]
pub struct Options {
    pub command: String,
    pub file: Option<String>,
    pub output: Option<String>,
    pub json: bool,
    pub strict: bool,
    pub release: bool,
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
            "-o" => opts.output = Some(args.next().ok_or("`-o` needs a file name")?),
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            other => {
                if opts.command.is_empty() {
                    opts.command = other.to_string();
                } else if opts.file.is_none() {
                    opts.file = Some(other.to_string());
                } else {
                    return Err(format!("unexpected argument `{other}`"));
                }
            }
        }
    }
    Ok(opts)
}
