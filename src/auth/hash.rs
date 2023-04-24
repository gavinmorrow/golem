use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashes password")
        .to_string();

    password_hash
}

/// Check if a password matches a hash.
///
/// # Panics
///
/// This function will panic if the hash is invalid.
pub fn check_passwords(password: String, hash: String) -> bool {
    let parsed_hash = PasswordHash::new(&hash).expect("hash is valid");
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
