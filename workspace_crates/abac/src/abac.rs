use db_manager::DbError;

pub async fn user_can_create_forum(db_pool: &sqlx::PgPool, user_id: String) -> Result<bool, DbError> {
    let answer = match sqlx::query!("SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
"user", user_id, "create_forum").fetch_optional(db_pool).await {
    Ok(opt) => opt, 
    Err(e) => {
        return Err(DbError::new(e));
    }
};

return Ok(answer.is_some());
    
}