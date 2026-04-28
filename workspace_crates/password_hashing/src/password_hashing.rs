use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use rand::{TryRng, rngs::SysRng};

pub const MAX_VALID_SALT_BYTES_LENGTH: usize = 48;
pub const SALT_BYTES_LENGTH: usize = MAX_VALID_SALT_BYTES_LENGTH;

fn generate_argon2_context_for_hashing(
    password_hasher_config: PasswordHasherConfig,
) -> Result<Argon2<'static>, String> {
    let PasswordHasherConfig {
        memory_cost_kib: m_cost,
        time_cost: t_cost,
        parallelism_cost: p_cost,
    } = password_hasher_config.clone();
    let params = Params::new(m_cost, t_cost, p_cost, None).map_err(|e| {
        dbg!(
            "Error on argo2 params::new",
            password_hasher_config,
            e.to_string()
        );
        e.to_string()
    })?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/*  I do not think verifying uses the m_cost and all that at all.
As far as I understand, it uses the parameters from the password string instead. So,
it makes no difference what context I use here. To avoid having to pass pointless params when calling password_verify, I guess I will do the thingie below...

*/
fn generate_argon2_context_for_verifying() -> Result<Argon2<'static>, String> {
    const DUMMY_M: u32 = 32000;
    const DUMMY_T_P: u32 = 1;
    generate_argon2_context_for_hashing(PasswordHasherConfig {
        memory_cost_kib: DUMMY_M,
        time_cost: DUMMY_T_P,
        parallelism_cost: DUMMY_T_P,
    })
}

#[derive(Debug, Clone)]
pub struct PasswordHasherConfigParams {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism_cost: u32,
}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PasswordHasherConfig {
    memory_cost_kib: u32,
    time_cost: u32,
    parallelism_cost: u32,
}
impl PasswordHasherConfig {
    pub fn new(params: PasswordHasherConfigParams) -> Self {
        Self {
            memory_cost_kib: params.memory_cost,
            time_cost: params.time_cost,
            parallelism_cost: params.parallelism_cost,
        }
    }
}

pub fn hash_password(
    password: String,
    hasher_config: PasswordHasherConfig,
) -> Result<String, String> {
    let salt = generate_salt(SALT_BYTES_LENGTH)?;
    let argon2 = generate_argon2_context_for_hashing(hasher_config)?;

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
    let argon2 = generate_argon2_context_for_verifying()?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_secure_random_bytes(length: usize) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; length];
    match SysRng.try_fill_bytes(&mut bytes) {
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
