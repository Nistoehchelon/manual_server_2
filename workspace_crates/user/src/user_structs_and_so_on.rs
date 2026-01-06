use db_manager::DbRow;

#[derive(Clone, Debug, Default)]
pub struct User {
    pub primary_key: i64,
    pub public_id: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub bio: String,
}

impl From<DbRow> for User {
    fn from(row: DbRow) -> Self {
        Self {
            primary_key: row.get("primary_key"),
            public_id: row.get("public_id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            password_salt: row.get("password_salt"),
            bio: row.get("bio"),
        }
    }
}
