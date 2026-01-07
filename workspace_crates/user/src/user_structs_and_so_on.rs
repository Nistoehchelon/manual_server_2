

#[derive(Clone, Debug, Default)]
pub struct User {
    pub primary_key: i64,
    pub public_id: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub bio: String,
}

