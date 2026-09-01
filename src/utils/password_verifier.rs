use argon2::{Argon2, PasswordHash, PasswordVerifier};
use bcrypt;

pub fn verify_password(password: &str, stored_hash: &str) -> Result<(), String> {
    // Detect bcrypt hash prefixes explicitly
    let is_bcrypt = stored_hash.starts_with("$2a$")
        || stored_hash.starts_with("$2b$")
        || stored_hash.starts_with("$2x$")
        || stored_hash.starts_with("$2y$");

    if is_bcrypt {
        // use the bcrypt crate directly, not jsonwebtoken::crypto::verify
        match bcrypt::verify(password, stored_hash) {
            Ok(true) => Ok(()),
            Ok(false) => Err("Invalid password".to_string()),
            Err(err) => Err(format!("Failed to verify bcrypt password: {err}")),
        }
    } else {
        // Argon2 verification
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|err| format!("Invalid password hash format: {err}"))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|err| format!("Invalid argon2 password: {err}"))?;
        Ok(())
    }
}
