//! `kode fmt`: a safe re-indenter. It only changes whitespace at the start and
//! end of lines and collapses blank lines, so it can never change what a
//! program means, and it keeps every comment.

struct Scan {
    brace_net: i32,
    paren_net: i32,
    leading_brace: i32,
    leading_paren: i32,
    ends_with_operator: bool,
}

const OPERATORS: &[&str] = &["+", "-", "*", "/", "%", "=", "==", "!=", "<", ">", "<=", ">=", ",", "??", "=>"];

fn scan(line: &str, in_block: &mut bool) -> Scan {
    let mut s = Scan {
        brace_net: 0,
        paren_net: 0,
        leading_brace: 0,
        leading_paren: 0,
        ends_with_operator: false,
    };
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut leading = true;
    let mut code = String::new();
    while i < chars.len() {
        let c = chars[i];
        if *in_block {
            if c == '*' && chars.get(i + 1) == Some(&'/') {
                *in_block = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        match c {
            '"' => {
                leading = false;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        break;
                    }
                    i += 1;
                }
                code.push('"');
                i += 1;
                continue;
            }
            '/' if chars.get(i + 1) == Some(&'/') => break,
            '/' if chars.get(i + 1) == Some(&'*') => {
                *in_block = true;
                i += 2;
                continue;
            }
            '{' => {
                s.brace_net += 1;
                leading = false;
            }
            '(' | '[' => {
                s.paren_net += 1;
                leading = false;
            }
            '}' => {
                s.brace_net -= 1;
                if leading {
                    s.leading_brace += 1;
                }
            }
            ')' | ']' => {
                s.paren_net -= 1;
                if leading {
                    s.leading_paren += 1;
                }
            }
            c if c.is_whitespace() => {
                i += 1;
                continue;
            }
            _ => leading = false,
        }
        code.push(c);
        i += 1;
    }
    let word_end = |w: &str| {
        code.ends_with(w)
            && code[..code.len() - w.len()]
                .chars()
                .last()
                .map(|p| !(p.is_alphanumeric() || p == '_'))
                .unwrap_or(true)
    };
    s.ends_with_operator = OPERATORS.iter().any(|o| code.ends_with(o)) || word_end("and") || word_end("or");
    s
}

pub fn format_source(src: &str) -> String {
    let mut out = String::new();
    let (mut depth, mut paren) = (0i32, 0i32);
    let mut in_block = false;
    let mut prev_continues = false;
    let mut pending_blank = false;

    for raw in src.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            pending_blank = !out.is_empty();
            continue;
        }
        if pending_blank {
            out.push('\n');
            pending_blank = false;
        }
        let started_in_comment = in_block;
        let s = scan(trimmed, &mut in_block);
        if started_in_comment {
            // inside a block comment: keep the line as it was
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let cont = i32::from(prev_continues && paren == 0);
        let levels = (depth - s.leading_brace).max(0) + (paren - s.leading_paren).max(0) + cont;
        out.push_str(&"    ".repeat(levels as usize));
        out.push_str(trimmed);
        out.push('\n');
        depth = (depth + s.brace_net).max(0);
        paren = (paren + s.paren_net).max(0);
        prev_continues = s.ends_with_operator;
    }
    out
}