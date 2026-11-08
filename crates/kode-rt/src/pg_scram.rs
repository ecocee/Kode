//! SCRAM-SHA-256 authentication (RFC 5802, RFC 7677) for PostgreSQL.
//!
//! PostgreSQL never sends the password: the server sends a salt and an iteration
//! count, the client returns a proof, and both sides check a signature they
//! derived from the stored key. The hash primitives live in `sha256.rs`.

use std::io::Read;

use crate::base64;
use crate::sha256::{hmac_sha256, pbkdf2_sha256, sha256};

/// Client state for one SCRAM-SHA-256 exchange (RFC 5802 / RFC 7677).
pub(crate) struct Scram {
    /// What we send first: `n,,n=,r=<nonce>`.
    pub(crate) client_first: String,
    /// Just `n=,r=<nonce>`, which is part of the auth message.
    first_bare: String,
    nonce: String,
    password: Vec<u8>,
    server_signature: [u8; 32],
}

fn client_nonce() -> String {
    let mut bytes = [0u8; 18];
    let random = std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .is_ok();
    if !random {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seed = format!("{}:{now}", std::process::id());
        bytes[..16].copy_from_slice(&sha256(seed.as_bytes())[..16]);
    }
    // `=` padding is not part of a nonce, and `,` would split the message.
    base64::encode(&bytes).trim_end_matches('=').to_string()
}

fn attribute(message: &str, key: char) -> Result<&str, String> {
    for part in message.split(',') {
        if let Some(value) = part.strip_prefix(key).and_then(|p| p.strip_prefix('=')) {
            return Ok(value);
        }
    }
    Err(format!("the server's SASL message has no `{key}=` field"))
}

impl Scram {
    pub(crate) fn start(password: &[u8]) -> Scram {
        let nonce = client_nonce();
        let first_bare = format!("n=,r={nonce}");
        Scram {
            client_first: format!("n,,{first_bare}"),
            first_bare,
            nonce,
            password: password.to_vec(),
            server_signature: [0; 32],
        }
    }

    /// Answer the server's `r=,s=,i=` challenge with `c=,r=,p=`.
    pub(crate) fn continue_with(&mut self, server_first: &str) -> Result<String, String> {
        let nonce = attribute(server_first, 'r')?;
        if !nonce.starts_with(&self.nonce) {
            return Err("the server's SCRAM nonce does not start with ours".into());
        }
        let salt = base64::decode(attribute(server_first, 's')?)?;
        let iterations: u32 = attribute(server_first, 'i')?
            .parse()
            .map_err(|_| "the server's SCRAM iteration count is not a number".to_string())?;
        if iterations == 0 {
            return Err("the server asked for zero SCRAM iterations".into());
        }

        let salted = pbkdf2_sha256(&self.password, &salt, iterations);
        let client_key = hmac_sha256(&salted, b"Client Key");
        let stored_key = sha256(&client_key);
        let without_proof = format!("c=biws,r={nonce}");
        let auth_message = format!("{},{server_first},{without_proof}", self.first_bare);
        let signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let proof: Vec<u8> = client_key
            .iter()
            .zip(signature.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        let server_key = hmac_sha256(&salted, b"Server Key");
        self.server_signature = hmac_sha256(&server_key, auth_message.as_bytes());
        Ok(format!("{without_proof},p={}", base64::encode(&proof)))
    }

    /// Check the server proved it knows the stored key too.
    pub(crate) fn finish(&self, server_final: &str) -> Result<(), String> {
        let signature = base64::decode(attribute(server_final, 'v')?)?;
        if signature != self.server_signature {
            return Err("the server's SCRAM signature does not match: the password is wrong or the connection was tampered with".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scram_matches_the_rfc_vector() {
        // RFC 7677 section 3, with the client nonce forced to the RFC's value.
        // The RFC puts the user name in `n=user`; PostgreSQL and libpq send an
        // empty name there (the startup message already named the role), which
        // is why the expected proof below is recomputed for `n=`.
        let mut state = Scram::start(b"pencil");
        state.nonce = "rOprNGfwEbeRWgbNEkqO".to_string();
        state.first_bare = "n=,r=rOprNGfwEbeRWgbNEkqO".to_string();
        state.client_first = format!("n,,{}", state.first_bare);
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let reply = state.continue_with(server_first).expect("continue");
        assert_eq!(
            reply,
            concat!(
                "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,",
                "p=qvT2SWdEH5Q06albL+hjSYuUhCG7VndFyzIb7CK4n9k="
            )
        );
        assert_eq!(
            base64::encode(&state.server_signature),
            "3HO6Qt1M4MKJrmlKaoOqLAI0/0TV0HZe7J9H3MBtSOg="
        );
        state
            .finish("v=3HO6Qt1M4MKJrmlKaoOqLAI0/0TV0HZe7J9H3MBtSOg=")
            .expect("the server proved it knows the key");
    }

    #[test]
    fn scram_rejects_a_wrong_nonce_or_signature() {
        let mut state = Scram::start(b"pencil");
        assert!(state
            .continue_with("r=someoneelse,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096")
            .is_err());
        assert!(state
            .continue_with("r=,s=W22ZaJ0SNY7soEsUEjb6gQ==")
            .is_err());
        state
            .finish("v=AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=")
            .expect_err("a wrong server signature must fail");
    }
}
