//! Reading PostgreSQL message bodies: rows, columns, command tags and errors.
//!
//! Every accessor is bounds-checked and returns an error instead of panicking on
//! a truncated message, so a half-closed connection cannot take the program down.

pub(crate) struct Column {
    pub name: String,
    pub oid: u32,
}

/// One result set: the command tag, the column list and the text rows.
#[derive(Default)]
pub(crate) struct Outcome {
    pub tag: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Option<Vec<u8>>>>,
    pub notice: Option<String>,
}

pub(crate) fn frame(kind: u8, body: &[u8], out: &mut Vec<u8>) {
    out.push(kind);
    out.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
    out.extend_from_slice(body);
}

pub(crate) fn printable(kind: u8) -> String {
    if kind.is_ascii_graphic() {
        (kind as char).to_string()
    } else {
        format!("{kind:#04x}")
    }
}

pub(crate) fn i32_at(body: &[u8], at: usize) -> Result<i32, String> {
    let slice = body
        .get(at..at + 4)
        .ok_or("the server sent a truncated message")?;
    Ok(i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

pub(crate) fn i16_at(body: &[u8], at: usize) -> Result<i16, String> {
    let slice = body
        .get(at..at + 2)
        .ok_or("the server sent a truncated message")?;
    Ok(i16::from_be_bytes([slice[0], slice[1]]))
}

pub(crate) fn cstr(body: &[u8], at: usize) -> Result<(&str, usize), String> {
    let rest = body
        .get(at..)
        .ok_or("the server sent a truncated message")?;
    let end = rest
        .iter()
        .position(|b| *b == 0)
        .ok_or("the server sent an unterminated string")?;
    let text = std::str::from_utf8(&rest[..end]).map_err(|_| "the server sent invalid UTF-8")?;
    Ok((text, at + end + 1))
}

pub(crate) fn parse_row_description(body: &[u8]) -> Result<Vec<Column>, String> {
    let count = i16_at(body, 0)?;
    if count < 0 {
        return Err("the server sent a negative column count".into());
    }
    let mut at = 2;
    let mut columns = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (name, next) = cstr(body, at)?;
        at = next;
        at += 4; // table oid
        at += 2; // column number
        let oid = i32_at(body, at)? as u32;
        at += 4;
        at += 2; // type length
        at += 4; // type modifier
        at += 2; // format code
        columns.push(Column {
            name: name.to_string(),
            oid,
        });
    }
    Ok(columns)
}

pub(crate) fn parse_data_row(body: &[u8]) -> Result<Vec<Option<Vec<u8>>>, String> {
    let count = i16_at(body, 0)?;
    if count < 0 {
        return Err("the server sent a negative value count".into());
    }
    let mut at = 2;
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let len = i32_at(body, at)?;
        at += 4;
        if len < 0 {
            values.push(None);
            continue;
        }
        let bytes = body
            .get(at..at + len as usize)
            .ok_or("the server sent a truncated row")?;
        values.push(Some(bytes.to_vec()));
        at += len as usize;
    }
    Ok(values)
}

/// Turn an ErrorResponse or NoticeResponse into one message, keeping the
/// SQLSTATE so callers can match on it.
pub(crate) fn server_error(body: &[u8]) -> String {
    let mut severity = String::new();
    let mut code = String::new();
    let mut message = String::new();
    let mut detail = String::new();
    let mut hint = String::new();
    // Each field is one byte naming it, then its text and a NUL.
    let mut at = 0;
    while let Some(&field) = body.get(at) {
        if field == 0 {
            break;
        }
        at += 1;
        let (value, next) = match cstr(body, at) {
            Ok(value) => value,
            Err(_) => break,
        };
        at = next;
        match field as char {
            'S' | 'V' => severity = value.to_string(),
            'C' => code = value.to_string(),
            'M' => message = value.to_string(),
            'D' => detail = value.to_string(),
            'H' => hint = value.to_string(),
            _ => {}
        }
    }
    let mut out = if code.is_empty() {
        message
    } else {
        format!("{code}: {message}")
    };
    if out.is_empty() {
        out = format!("the server sent an error ({})", printable(b'E'));
    }
    if !severity.is_empty() && severity != "ERROR" && severity != "FATAL" && severity != "PANIC" {
        out = format!("{severity}: {out}");
    }
    if !detail.is_empty() {
        out.push_str("\ndetail: ");
        out.push_str(&detail);
    }
    if !hint.is_empty() {
        out.push_str("\nhint: ");
        out.push_str(&hint);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_framing() {
        let mut out = Vec::new();
        frame(b'Q', b"select 1\0", &mut out);
        assert_eq!(out[0], b'Q');
        assert_eq!(i32::from_be_bytes([out[1], out[2], out[3], out[4]]), 13);
        assert_eq!(&out[5..], b"select 1\0");
    }

    #[test]
    fn row_description_is_parsed() {
        let mut body = Vec::new();
        body.extend_from_slice(&1i16.to_be_bytes());
        body.extend_from_slice(b"id\0");
        body.extend_from_slice(&0i32.to_be_bytes());
        body.extend_from_slice(&0i16.to_be_bytes());
        body.extend_from_slice(&23i32.to_be_bytes());
        body.extend_from_slice(&4i16.to_be_bytes());
        body.extend_from_slice(&(-1i32).to_be_bytes());
        body.extend_from_slice(&0i16.to_be_bytes());
        let columns = parse_row_description(&body).expect("parse");
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].name, "id");
        assert_eq!(columns[0].oid, 23);

        let mut row = Vec::new();
        row.extend_from_slice(&2i16.to_be_bytes());
        row.extend_from_slice(&2i32.to_be_bytes());
        row.extend_from_slice(b"42");
        row.extend_from_slice(&(-1i32).to_be_bytes());
        let values = parse_data_row(&row).expect("parse");
        assert_eq!(values[0].as_deref(), Some(&b"42"[..]));
        assert_eq!(values[1], None);
    }

    #[test]
    fn truncated_messages_are_errors() {
        assert!(parse_row_description(&[0, 1, 2]).is_err());
        assert!(parse_data_row(&[0, 1, 0, 0, 0]).is_err());
        assert!(cstr(b"no terminator", 0).is_err());
    }

    #[test]
    fn server_error_keeps_the_code() {
        let body =
            b"SERROR\0C42P01\0Mrelation \"nope\" does not exist\0Ddetail\0Hcreate it first\0\0";
        let message = server_error(body);
        assert!(message.starts_with("42P01: relation \"nope\" does not exist"));
        assert!(message.contains("detail: detail"));
        assert!(message.contains("hint: create it first"));

        // A notice keeps its severity so it reads as a notice, not a failure.
        let notice = server_error(b"SNOTICE\0C00000\0Mheads up\0");
        assert_eq!(notice, "NOTICE: 00000: heads up");
    }
}
