use db_manager::DbRow;

#[derive(Clone, Debug, Default)]
pub struct User {
    pub internal_id: i64,
    pub public_inapp_id: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub bio: String,
    pub role_id: i64,
}

impl From<DbRow> for User {
    fn from(row: DbRow) -> Self {
        Self {
            internal_id: row.get("internal_id"),
            public_inapp_id: row.get("public_inapp_id"),
            username: row.get("username"),
            password_hash: row.get("password_hash"),
            password_salt: row.get("password_salt"),
            bio: row.get("bio"),
            role_id: row.get("role_id"),
        }
    }
}
