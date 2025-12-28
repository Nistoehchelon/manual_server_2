use hex;
use rand::RngCore;

pub const SALT_BYTES_LENGTH: usize = 64;

pub fn generate_secure_random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

pub struct HashAndSalt {
    pub hash: String,
    pub salt: String,
}

pub fn verify_password(
    unhashed_unconfirmed_password: String,
    the_hash: String,
    the_salt: String,
) -> bool {
    let hash_of_unconfirmed = hash(format!("{unhashed_unconfirmed_password}{the_salt}"));
    return hash_of_unconfirmed == the_hash;
}

pub fn hash_password(unhashed_password: String) -> HashAndSalt {
    let salt = generate_salt();
    let hash_salt_string = format!("{}{}", unhashed_password, salt);
    let hash = hash(hash_salt_string);
    return HashAndSalt { hash, salt };
}

fn hash(string_i_want_to_hash: String) -> String {
    let hash = blake3::hash(string_i_want_to_hash.as_bytes());
    let hashed_string = hash.to_string();
    return hashed_string;
}

fn generate_salt() -> String {
    let bytes = generate_secure_random_bytes(SALT_BYTES_LENGTH);
    let hexed_bytes = encode_bytes_to_hex_string(bytes);
    return hexed_bytes;
}

fn encode_bytes_to_hex_string(bytes: Vec<u8>) -> String {
    return hex::encode(bytes);
}
