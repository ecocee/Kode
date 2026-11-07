//! Recursive-descent JSON parser for `json_parse`.
//!
//! Strict RFC 8259 syntax, with a depth limit so hostile input cannot exhaust the
//! stack. On an error, any half-built list or map is released before returning, so
//! a caught parse error never leaks.

use crate::json::{make_bool, make_float, make_int, make_null, make_string, wrap_list, wrap_map};
use crate::list::{kode_list_new, kode_list_push};
use crate::map::{kode_map_new, kode_map_set};
use crate::object::{kode_release, Header};
use crate::string::from_bytes;

const MAX_DEPTH: usize = 256;

pub(crate) fn parse(text: &str) -> Result<*mut Header, String> {
    let bytes = text.as_bytes();
    let start = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        0
    };
    let mut p = Parser {
        s: bytes,
        i: start,
        depth: 0,
    };
    p.ws();
    let value = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        unsafe { kode_release(value) };
        return Err(p.err("unexpected text after the JSON value"));
    }
    Ok(value)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        Some(b)
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.i += 1;
        }
    }

    fn err(&self, msg: &str) -> String {
        let pos = self.i.min(self.s.len());
        let line = 1 + self.s[..pos].iter().filter(|&&b| b == b'\n').count();
        let column = 1 + self.s[..pos]
            .iter()
            .rev()
            .take_while(|&&b| b != b'\n')
            .count();
        format!("invalid JSON at line {line}, column {column}: {msg}")
    }

    fn expect(&mut self, byte: u8, what: &str) -> Result<(), String> {
        match self.bump() {
            Some(b) if b == byte => Ok(()),
            _ => Err(self.err(what)),
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), String> {
        for expected in word.bytes() {
            if self.bump() != Some(expected) {
                return Err(self.err(&format!("expected `{word}`")));
            }
        }
        Ok(())
    }

    fn value(&mut self) -> Result<*mut Header, String> {
        if self.depth >= MAX_DEPTH {
            return Err(self.err("the JSON is nested too deeply"));
        }
        let b = self.peek().ok_or_else(|| self.err("expected a value"))?;
        self.depth += 1;
        let result = match b {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => {
                let s = self.string()?;
                Ok(unsafe { make_string(from_bytes(s.as_bytes())) })
            }
            b't' => {
                self.literal("true")?;
                Ok(unsafe { make_bool(true) })
            }
            b'f' => {
                self.literal("false")?;
                Ok(unsafe { make_bool(false) })
            }
            b'n' => {
                self.literal("null")?;
                Ok(unsafe { make_null() })
            }
            b'-' | b'0'..=b'9' => self.number(),
            _ => Err(self.err("expected a value")),
        };
        self.depth -= 1;
        result
    }

    fn object(&mut self) -> Result<*mut Header, String> {
        self.bump(); // {
        let map = kode_map_new(1);
        let result = self.object_body(map);
        if result.is_err() {
            unsafe { kode_release(map) };
        }
        result
    }

    fn object_body(&mut self, map: *mut Header) -> Result<*mut Header, String> {
        self.ws();
        if self.peek() == Some(b'}') {
            self.bump();
            return Ok(unsafe { wrap_map(map) });
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(self.err("expected a string key"));
            }
            let key = self.string()?;
            self.ws();
            self.expect(b':', "expected `:` after the key")?;
            self.ws();
            let value = self.value()?;
            unsafe {
                let k = from_bytes(key.as_bytes());
                kode_map_set(map, k, value as i64);
                kode_release(k);
            }
            self.ws();
            match self.peek() {
                Some(b',') => {
                    self.bump();
                }
                Some(b'}') => {
                    self.bump();
                    break;
                }
                _ => return Err(self.err("expected `,` or `}`")),
            }
        }
        Ok(unsafe { wrap_map(map) })
    }

    fn array(&mut self) -> Result<*mut Header, String> {
        self.bump(); // [
        let list = kode_list_new(1);
        let result = self.array_body(list);
        if result.is_err() {
            unsafe { kode_release(list) };
        }
        result
    }

    fn array_body(&mut self, list: *mut Header) -> Result<*mut Header, String> {
        self.ws();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(unsafe { wrap_list(list) });
        }
        loop {
            self.ws();
            let value = self.value()?;
            unsafe { kode_list_push(list, value as i64) };
            self.ws();
            match self.peek() {
                Some(b',') => {
                    self.bump();
                }
                Some(b']') => {
                    self.bump();
                    break;
                }
                _ => return Err(self.err("expected `,` or `]`")),
            }
        }
        Ok(unsafe { wrap_list(list) })
    }

    fn string(&mut self) -> Result<String, String> {
        self.bump(); // opening quote
        let mut out = String::new();
        loop {
            let b = self
                .bump()
                .ok_or_else(|| self.err("this string is never closed"))?;
            match b {
                b'"' => break,
                b'\\' => self.escape(&mut out)?,
                0x00..=0x1f => {
                    return Err(self.err("a control character must be escaped in a string"))
                }
                0x20..=0x7f => out.push(b as char),
                _ => {
                    let start = self.i - 1;
                    let len = utf8_len(b).ok_or_else(|| self.err("invalid UTF-8 in a string"))?;
                    let end = start + len;
                    if end > self.s.len() {
                        return Err(self.err("invalid UTF-8 in a string"));
                    }
                    let chunk = std::str::from_utf8(&self.s[start..end])
                        .map_err(|_| self.err("invalid UTF-8 in a string"))?;
                    out.push_str(chunk);
                    self.i = end;
                }
            }
        }
        Ok(out)
    }

    fn escape(&mut self, out: &mut String) -> Result<(), String> {
        let b = self
            .bump()
            .ok_or_else(|| self.err("this string is never closed"))?;
        match b {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{8}'),
            b'f' => out.push('\u{c}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let high = self.hex4()?;
                let code = if (0xD800..=0xDBFF).contains(&high) {
                    if self.bump() != Some(b'\\') || self.bump() != Some(b'u') {
                        return Err(
                            self.err("a high surrogate must be followed by a low surrogate")
                        );
                    }
                    let low = self.hex4()?;
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        return Err(self
                            .err("the second half of the surrogate pair is not a low surrogate"));
                    }
                    0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
                } else if (0xDC00..=0xDFFF).contains(&high) {
                    return Err(self.err("a low surrogate must follow a high surrogate"));
                } else {
                    high
                };
                let c = char::from_u32(code)
                    .ok_or_else(|| self.err("this is not a Unicode character"))?;
                out.push(c);
            }
            _ => return Err(self.err("unknown escape in a string")),
        }
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        for _ in 0..4 {
            let b = self
                .bump()
                .ok_or_else(|| self.err("this string is never closed"))?;
            let digit = (b as char)
                .to_digit(16)
                .ok_or_else(|| self.err("`\\u` needs four hexadecimal digits"))?;
            v = v * 16 + digit;
        }
        Ok(v)
    }

    fn number(&mut self) -> Result<*mut Header, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.bump();
        }
        match self.peek() {
            Some(b'0') => {
                self.bump();
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.err("a number cannot start with 0"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.bump();
                }
            }
            _ => return Err(self.err("expected a digit")),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.bump();
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("expected a digit after `.`"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.bump();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.bump();
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("expected a digit in the exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.bump();
            }
        }
        let text = unsafe { std::str::from_utf8_unchecked(&self.s[start..self.i]) };
        if is_float {
            let v: f64 = text
                .parse()
                .map_err(|_| self.err("this is not a valid float"))?;
            if !v.is_finite() {
                return Err(self.err("this float is out of range"));
            }
            Ok(unsafe { make_float(v) })
        } else {
            let v: i64 = text
                .parse()
                .map_err(|_| self.err("this integer is too large for a 64-bit int"))?;
            Ok(unsafe { make_int(v) })
        }
    }
}

fn utf8_len(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7f => Some(1),
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}
