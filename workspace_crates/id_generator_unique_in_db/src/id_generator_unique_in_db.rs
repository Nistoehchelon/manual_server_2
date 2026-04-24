use db_manager::DbError;
use sqlx::{Postgres, Transaction};

pub async fn generate_unique_user_id(tx: &mut Transaction<'_, Postgres>) -> Result<String, DbError> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_random_alphanumeric_id(32);

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(DbError::new)?;

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new(
        "Failed to generate a unique ID after 20 attempts",
    ))
}



pub async fn generate_unique_thread_id(db_pool: sqlx::PgPool) -> Result<String, DbError> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_unique_id();

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM forum_threads WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&db_pool)
        .await
        .map_err(DbError::new)?;

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new(
        "Failed to generate a unique thread ID after 20 attempts",
    ))
}

pub async fn generate_unique_forum_id(db_pool: sqlx::PgPool) -> Result<String, DbError> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_unique_id();

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM forums WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&db_pool)
        .await
        .map_err(DbError::new)?;

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new(
        "Failed to generate a unique thread ID after 20 attempts",
    ))
}

pub async fn generate_unique_forum_thread_message_id(tx: &mut Transaction<'_, Postgres>) -> Result<String, DbError> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_unique_id();

        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM forum_thread_messages WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(DbError::new)?;

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new(
        "Failed to generate a unique thread ID after 20 attempts",
    ))
}
