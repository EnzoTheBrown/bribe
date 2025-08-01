use argon2::{
    password_hash::{
        rand_core::OsRng, Error as PwError, PasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};

pub fn hash_password(plaintext: &str) -> Result<String, PwError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(plaintext.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(plaintext: &str, hash_str: &str) -> Result<bool, PwError> {
    let parsed = PasswordHash::new(hash_str)?;
    Ok(Argon2::default()
        .verify_password(plaintext.as_bytes(), &parsed)
        .is_ok())
}
