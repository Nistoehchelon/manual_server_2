use db_manager::DbError;
use sqlx::{Postgres, Transaction};

// I can use a non-macro sqlx::query_scalar or something, but I don't like the idea of silently finding out
// error only on runtime... Not sure if the match case below is worth it, though...
pub async fn generate_unique_id_for(
    tx: &mut Transaction<'_, Postgres>,
    for_table: &str,
) -> Result<String, DbError> {
    let allowed = ["users", "forum_threads", "forum_thread_messages"];
    if !allowed.contains(&for_table) {
        return Err(DbError::new("Invalid table name".to_string()));
    }
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_random_alphanumeric_id(32);

        let exists = match for_table {
            "users" => sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
                id
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::new)?,
            "forum_threads" => sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM forum_threads WHERE id = $1) as "exists!""#,
                id
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::new)?,
            "forum_thread_messages" => sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM forum_thread_messages WHERE id = $1) as "exists!""#,
                id
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::new)?,
            _ => return Err(DbError::new("Invalid table name")),
        };

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new(
        "Failed to generate a unique ID after 20 attempts",
    ))
}
pub async fn generate_unique_user_id(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, DbError> {
    generate_unique_id_for(tx, "users").await
}

pub async fn generate_unique_thread_id(tx: &mut Transaction<'_, Postgres>) -> Result<String, DbError> {
   generate_unique_id_for(tx, "forum_threads").await
}

pub async fn generate_unique_forum_id(tx: &mut Transaction<'_, Postgres>) -> Result<String, DbError> {
   generate_unique_id_for(tx, "forums").await
   }

pub async fn generate_unique_forum_thread_message_id(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, DbError> {
   generate_unique_id_for(tx, "forum_thread_messages").await
}
