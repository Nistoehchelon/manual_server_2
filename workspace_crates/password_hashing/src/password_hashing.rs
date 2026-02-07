use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use rand::{TryRngCore, rand_core::OsRng};

pub const MAX_VALID_SALT_BYTES_LENGTH: usize = 48;
pub const SALT_BYTES_LENGTH: usize = MAX_VALID_SALT_BYTES_LENGTH;

fn strong_argon2() -> Result<Argon2<'static>, String> {
    let m_cost = 32 * 1024;
    let t_cost = 2;
    let p_cost = 2;
    let params = Params::new(m_cost, t_cost, p_cost, None).map_err(|e| {
        dbg!("Error on argo2 params::new", e.to_string());
        e.to_string()
    })?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn hash_password(password: String) -> Result<String, String> {
    let salt = generate_salt(SALT_BYTES_LENGTH)?;
    let argon2 = strong_argon2()?;

    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(ph) => ph.to_string(),
        Err(e) => {
            dbg!("Error hashing password", e);
            return Err(e.to_string());
        }
    };

    Ok(password_hash)
}

pub fn verify_password(password: String, stored_hash: String) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(&stored_hash).map_err(|e| {
        dbg!("Error parsing hash", e.to_string());
        e.to_string()
    })?;
    let argon2 = strong_argon2()?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_secure_random_bytes(length: usize) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; length];
    match OsRng.try_fill_bytes(&mut bytes) {
        Ok(_) => {}
        Err(e) => {
            return Err(e.to_string());
        }
    };
    Ok(bytes)
}
pub fn generate_salt(length: usize) -> Result<SaltString, String> {
    SaltString::encode_b64(&generate_secure_random_bytes(length)?).map_err(|e| {
            dbg!("Error generating salt", e.to_string());
            e.to_string()
        })
}
