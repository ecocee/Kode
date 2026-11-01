use kode_diagnostics::{Diagnostic, Severity};

use crate::source::SourceMap;

/// Show diagnostics (as text on stderr, or JSON on stdout).
/// Returns true when any of them is an error.
pub fn report(json: bool, map: &SourceMap, diags: &[Diagnostic]) -> bool {
    if diags.is_empty() {
        return false;
    }
    if json {
        let items: Vec<String> = diags.iter().map(|d| map.json(d)).collect();
        println!("[{}]", items.join(","));
    } else {
        for d in diags {
            eprintln!("{}", map.render(d));
        }
    }
    diags.iter().any(|d| d.severity == Severity::Error)
}