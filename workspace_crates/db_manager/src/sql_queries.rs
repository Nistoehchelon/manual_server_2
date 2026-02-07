use crate::{DbError, DbPool};

pub async fn get_token_id_by_session_id_and_designated_purpose(
    executor: DbPool,
    session_id: String,
    designated_purpose: String,
) -> Result<Option<String>, DbError> {
    match sqlx::query!(
        "SELECT id FROM csrf_tokens WHERE session_id = $1 AND designated_purpose = $2",
        session_id,
        designated_purpose
    )
    .fetch_optional(&executor)
    .await
    {
        Ok(opt) => match opt {
            Some(s) => Ok(Some(s.id)),
            None => {
                Ok(None)
            }
        },
        Err(e) => {
            Err(DbError::new(e))
        }
    }
}

pub async fn add_new_token(
    executor: DbPool,
    token_id: String,
    designated_purpose: String,
    session_id: String,
) -> Result<(), DbError> {
    match sqlx::query!(
        "INSERT INTO csrf_tokens (id,designated_purpose, session_id) VALUES ($1,$2,$3)",
        token_id,
        designated_purpose,
        session_id
    )
    .execute(&executor)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(DbError::new(e));
        }
    };
    Ok(())
}

pub async fn validate_token(
    executor: DbPool,
    session_id: String,
    token_id: String,
) -> Result<bool, DbError> {
    match sqlx::query!(
        "SELECT id FROM csrf_tokens WHERE session_id = $1 AND id = $2",
        session_id,
        token_id
    )
    .fetch_optional(&executor)
    .await
    {
        Ok(opt) => {
            if opt.is_some() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            Err(DbError::new(e))
        }
    }
}

pub async fn user_has_create_forum_permission(
    pool: DbPool,
    user_id: String,
) -> Result<bool, DbError> {
    match sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        "user",
        user_id,
        "create_forum"
    )
    .fetch_optional(&pool)
    .await
    .map_err(DbError::new)?
    {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
