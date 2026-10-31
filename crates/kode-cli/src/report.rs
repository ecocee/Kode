use kode_diagnostics::{Diagnostic, Severity};

/// Show diagnostics (as text on stderr, or JSON on stdout).
/// Returns true when any of them is an error.
pub fn report(json: bool, file: &str, src: &str, diags: &[Diagnostic]) -> bool {
    if diags.is_empty() {
        return false;
    }
    if json {
        let items: Vec<String> = diags.iter().map(|d| d.to_json(file, src)).collect();
        println!("[{}]", items.join(","));
    } else {
        for d in diags {
            eprintln!("{}", d.render(file, src));
        }
    }
    diags.iter().any(|d| d.severity == Severity::Error)
}
