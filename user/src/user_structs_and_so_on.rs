


#[derive(Clone, Debug, Default, postgres_types::FromSql, postgres_types::ToSql )]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String, 
    pub bio: String,
    pub role_id: i64,
}

