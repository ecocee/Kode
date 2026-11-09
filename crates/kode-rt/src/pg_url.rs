//! The PostgreSQL connection string: `postgres://user:password@host:port/database`.
//!
//! Follows libpq where it matters: the database defaults to the user name, the
//! port defaults to 5432, an empty host means `127.0.0.1`, and percent escapes
//! are decoded in the user name, password and database. Options that this client
//! cannot honour (a required TLS handshake, unknown keys) are errors rather than
//! silently ignored.

use std::net::{SocketAddr, ToSocketAddrs};

const MAX_STARTUP_PARAM: usize = 64 << 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ssl {
    /// Send an SSLRequest and upgrade to TLS when the server offers it.
    Prefer,
    /// Never ask for TLS (the server still has to allow plaintext).
    Disable,
}

pub(crate) struct Target {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub application_name: String,
    pub ssl: Ssl,
}

fn percent_decode(text: &str, what: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes
                .get(i + 1..i + 3)
                .ok_or_else(|| format!("the {what} ends with an incomplete % escape"))?;
            let hex = std::str::from_utf8(hex)
                .map_err(|_| format!("the {what} has an invalid % escape"))?;
            let byte = u8::from_str_radix(hex, 16)
                .map_err(|_| format!("the {what} has an invalid % escape"))?;
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| format!("the {what} is not valid UTF-8"))
}

fn parse_port(text: &str) -> Result<u16, String> {
    let port: u32 = text
        .parse()
        .map_err(|_| format!("`{text}` is not a valid port"))?;
    if port == 0 || port > 65535 {
        return Err(format!("port {port} is out of range"));
    }
    Ok(port as u16)
}

/// `postgres://user:password@host:port/database?sslmode=disable`
pub(crate) fn parse_url(url: &str) -> Result<Target, String> {
    let text = url.trim();
    if text.is_empty() {
        return Err(
            "the connection string is empty; use postgres://user:password@host:port/database"
                .into(),
        );
    }
    let (scheme, rest) = text
        .split_once("://")
        .ok_or("the connection string must start with postgres:// or postgresql://")?;
    if !scheme.eq_ignore_ascii_case("postgres") && !scheme.eq_ignore_ascii_case("postgresql") {
        return Err(format!(
            "unsupported connection scheme `{scheme}`: use postgres://"
        ));
    }
    let (head, query) = match rest.split_once('?') {
        Some((h, q)) => (h, Some(q)),
        None => (rest, None),
    };
    let (authority, path) = match head.split_once('/') {
        Some((a, p)) => (a, Some(p)),
        None => (head, None),
    };
    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((u, h)) => (Some(u), h),
        None => (None, authority),
    };

    let (mut user, mut password) = (String::new(), String::new());
    if let Some(info) = userinfo {
        let (u, p) = match info.split_once(':') {
            Some((u, p)) => (u, Some(p)),
            None => (info, None),
        };
        user = percent_decode(u, "user name")?;
        password = percent_decode(p.unwrap_or(""), "password")?;
    }

    let (host, port) = if let Some(rest) = hostport.strip_prefix('[') {
        let end = rest.find(']').ok_or("the IPv6 host has no closing `]`")?;
        let host = rest[..end].to_string();
        let after = &rest[end + 1..];
        let port = match after.strip_prefix(':') {
            Some(p) => parse_port(p)?,
            None if after.is_empty() => 5432,
            _ => return Err("unexpected text after the IPv6 host".into()),
        };
        (host, port)
    } else {
        match hostport.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), parse_port(p)?),
            None => (hostport.to_string(), 5432),
        }
    };
    let host = if host.is_empty() {
        "127.0.0.1".to_string()
    } else {
        host
    };

    let database = percent_decode(path.unwrap_or(""), "database name")?;
    // Like libpq: the database defaults to the user name.
    let database = if database.is_empty() {
        user.clone()
    } else {
        database
    };

    let mut ssl = Ssl::Prefer;
    let mut application_name = "kode".to_string();
    if let Some(query) = query {
        for pair in query.split('&').filter(|p| !p.is_empty()) {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match key {
                "sslmode" => {
                    ssl = match value {
                        "disable" => Ssl::Disable,
                        "prefer" | "allow" => Ssl::Prefer,
                        "require" | "verify-ca" | "verify-full" => {
                            return Err(
                                "sslmode=require is not supported yet: use sslmode=prefer, which \
                                 upgrades to TLS when the server offers it, or terminate TLS in a \
                                 local proxy"
                                    .into(),
                            )
                        }
                        other => return Err(format!("unknown sslmode `{other}`")),
                    }
                }
                "application_name" => application_name = percent_decode(value, "application_name")?,
                other => {
                    return Err(format!(
                        "unsupported connection option `{other}`: Kode understands sslmode and application_name"
                    ))
                }
            }
        }
    }

    for (what, value) in [
        ("user name", &user),
        ("password", &password),
        ("database name", &database),
    ] {
        if value.contains('\0') {
            return Err(format!("the {what} contains a NUL byte"));
        }
        if value.len() > MAX_STARTUP_PARAM {
            return Err(format!("the {what} is too long"));
        }
    }
    if host.contains('\0') {
        return Err("the host contains a NUL byte".into());
    }

    Ok(Target {
        host,
        port,
        user,
        password,
        database,
        application_name,
        ssl,
    })
}

pub(crate) fn resolve(host: &str, port: u16) -> Result<SocketAddr, String> {
    (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("cannot resolve {host}: {e}"))?
        .next()
        .ok_or_else(|| format!("{host} has no address"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_url() {
        let t = parse_url("postgres://bench:secret@127.0.0.1:55432/kode_bench").expect("parse");
        assert_eq!(t.host, "127.0.0.1");
        assert_eq!(t.port, 55432);
        assert_eq!(t.user, "bench");
        assert_eq!(t.password, "secret");
        assert_eq!(t.database, "kode_bench");
        assert_eq!(t.ssl, Ssl::Prefer);
        assert_eq!(t.application_name, "kode");
    }

    #[test]
    fn defaults_and_options() {
        let t = parse_url("postgresql://alice@db.internal/app?sslmode=disable").expect("parse");
        assert_eq!(t.host, "db.internal");
        assert_eq!(t.port, 5432);
        assert_eq!(t.password, "");
        assert_eq!(t.database, "app");
        assert_eq!(t.ssl, Ssl::Disable);

        let t = parse_url("postgres://bob@localhost").expect("parse");
        assert_eq!(t.database, "bob", "the database defaults to the user");

        let t = parse_url("postgres://[::1]:6000/x").expect("parse");
        assert_eq!(t.host, "::1");
        assert_eq!(t.port, 6000);

        let t = parse_url("postgres://[::1]/x").expect("parse");
        assert_eq!(t.port, 5432);
    }

    #[test]
    fn percent_escapes() {
        let t = parse_url("postgres://user%40host:p%40ss@127.0.0.1:5432/db%20name").expect("parse");
        assert_eq!(t.user, "user@host");
        assert_eq!(t.password, "p@ss");
        assert_eq!(t.database, "db name");
    }

    #[test]
    fn bad_urls_are_explained() {
        assert!(parse_url("").is_err());
        assert!(error_of("mysql://x/y").contains("scheme"));
        assert!(parse_url("postgres://h:0/db").is_err());
        assert!(parse_url("postgres://h:99999/db").is_err());
        assert!(error_of("postgres://h/db?sslmode=require").contains("TLS"));
        assert!(parse_url("postgres://h/db?sslmode=maybe").is_err());
        assert!(error_of("postgres://h/db?mode=x").contains("mode"));
        assert!(error_of("postgres://h/db%zz").contains("%"));
    }

    /// The message of a bad connection string, without requiring `Target: Debug`
    /// (whose derived output would print the password).
    fn error_of(url: &str) -> String {
        match parse_url(url) {
            Ok(_) => panic!("{url} should not parse"),
            Err(message) => message,
        }
    }
}
