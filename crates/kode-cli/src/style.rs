//! Colour and layout for what the CLI prints.
//!
//! Colour is used only when the stream is a terminal and the environment asks
//! for it, so piping into a file, a test or an editor never sees escape codes.

use std::io::IsTerminal;
use std::time::Duration;

/// `Some(true)` / `Some(false)` when the environment decides, `None` when the
/// stream decides.
fn forced() -> Option<bool> {
    if std::env::var_os("CLICOLOR_FORCE").is_some() || std::env::var_os("FORCE_COLOR").is_some() {
        return Some(true);
    }
    if std::env::var_os("NO_COLOR").is_some()
        || std::env::var_os("TERM").is_some_and(|t| t == "dumb")
    {
        return Some(false);
    }
    None
}

pub fn on_stdout() -> bool {
    forced().unwrap_or_else(|| std::io::stdout().is_terminal())
}

pub fn on_stderr() -> bool {
    forced().unwrap_or_else(|| std::io::stderr().is_terminal())
}

pub fn paint(on: bool, code: &str, text: &str) -> String {
    if on {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(on: bool, text: &str) -> String {
    paint(on, "1", text)
}

pub fn dim(on: bool, text: &str) -> String {
    paint(on, "2", text)
}

pub fn red(on: bool, text: &str) -> String {
    paint(on, "31", text)
}

pub fn green(on: bool, text: &str) -> String {
    paint(on, "32", text)
}

pub fn yellow(on: bool, text: &str) -> String {
    paint(on, "33", text)
}

pub fn cyan(on: bool, text: &str) -> String {
    paint(on, "36", text)
}

/// A duration that reads well from a single token to a few seconds.
pub fn millis(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{ms:.2} ms")
    } else if ms < 1000.0 {
        format!("{ms:.0} ms")
    } else {
        format!("{:.2} s", ms / 1000.0)
    }
}

/// Colour each line of a rendered diagnostic by how it starts. The text itself
/// is untouched, so the plain form is the same bytes.
pub fn diagnostic(text: &str, on: bool) -> String {
    if !on {
        return text.to_string();
    }
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if line.starts_with("error") {
                red(true, line)
            } else if line.starts_with("warning") {
                yellow(true, line)
            } else if trimmed.starts_with("help:") {
                cyan(true, line)
            } else if trimmed.starts_with("note:") || trimmed.starts_with("-->") {
                dim(true, line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
