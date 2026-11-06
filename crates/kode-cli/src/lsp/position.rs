//! Source positions: byte offsets in a file <-> LSP `{line, character}` (UTF-16),
//! plus `file://` URI conversion.

use std::path::PathBuf;

/// Byte offset -> (line, UTF-16 column), both zero based.
pub fn position(text: &str, offset: usize) -> (u32, u32) {
    let mut off = offset.min(text.len());
    while !text.is_char_boundary(off) {
        off -= 1;
    }
    let (mut line, mut col) = (0u32, 0u32);
    for ch in text[..off].chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    (line, col)
}

/// (line, UTF-16 column) -> byte offset.
pub fn offset(text: &str, line: u32, character: u32) -> usize {
    let mut start = 0usize;
    if line > 0 {
        let mut seen = 0u32;
        let mut found = false;
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                seen += 1;
                if seen == line {
                    start = i + 1;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return text.len();
        }
    }
    let mut units = 0u32;
    for (i, ch) in text[start..].char_indices() {
        if units >= character || ch == '\n' {
            return start + i;
        }
        units += ch.len_utf16() as u32;
    }
    text.len()
}

/// The identifier or word around `at` (byte offsets).
pub fn word_at(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let start = text[..at].char_indices().rev().find(|(_, c)| !is_word(*c)).map(|(i, c)| i + c.len_utf8()).unwrap_or(0);
    let end = text[at..].char_indices().find(|(_, c)| !is_word(*c)).map(|(i, _)| at + i).unwrap_or(text.len());
    (start, end)
}

pub fn path_from_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    let mut path = percent_decode(rest);
    // Windows URIs look like /C:/Users/...; the leading slash is not part of the path.
    if cfg!(windows) && path.len() > 2 && path.as_bytes()[0] == b'/' && path.as_bytes()[2] == b':' {
        path.remove(0);
    }
    Some(PathBuf::from(path))
}

/// Path -> `file://` URI. Kept for features that need to name another file
/// (go-to-definition), and covered by the tests below.
#[allow(dead_code)]
pub fn uri_from_path(path: &std::path::Path) -> String {
    let text = path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).display().to_string();
    let mut out = String::from("file://");
    for b in text.bytes() {
        match b {
            b'/' | b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':' => out.push(b as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_offsets_and_positions_both_ways() {
        let text = "a = 1\nb = \"héllo\"\nprint(b)\n";
        let at = text.find("print").unwrap();
        let (line, col) = position(text, at);
        assert_eq!((line, col), (2, 0));
        assert_eq!(offset(text, line, col), at);
        let hello = text.find("héllo").unwrap() + "hé".len();
        let (l, c) = position(text, hello);
        assert_eq!(l, 1);
        assert_eq!(offset(text, l, c), hello);
    }

    #[test]
    fn round_trips_file_uris() {
        let path = std::env::temp_dir().join("kode lsp test.kode");
        let uri = uri_from_path(&path);
        assert!(uri.starts_with("file://"));
        let back = path_from_uri(&uri).unwrap();
        assert_eq!(back.file_name(), path.file_name());
    }

    #[test]
    fn finds_the_word_under_a_cursor() {
        let text = "print(total)";
        let (start, end) = word_at(text, 8);
        assert_eq!(&text[start..end], "total");
    }
}
