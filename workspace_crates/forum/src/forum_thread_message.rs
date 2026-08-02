use error_structs::SomeSortOfError;
use tracing::debug;

use crate::ForumThreadMessage;

pub async fn edit_forum_thread_message(
    db_pool: sqlx::PgPool,
    msg_id: String,
    new_content: String,
    user_id: String,
) -> Result<(), SomeSortOfError> {
    sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3)", user_id.clone(), new_content, msg_id).execute(&db_pool).await.map_err(SomeSortOfError::new)?;

    Ok(())
}

pub async fn get_message_page(
    db_pool: sqlx::PgPool,
    msg_id: String,
    msgs_per_page: u64,
) -> Result<i64, SomeSortOfError> {
    let msg = get_message(db_pool.clone(), msg_id.clone())
        .await
        .map_err(SomeSortOfError::new)?;
    let thread_id = msg.thread_id.clone();

    let row = sqlx::query!(
        "SELECT position FROM ( SELECT id, ROW_NUMBER() OVER (ORDER BY primary_key ASC) as position FROM forum_thread_messages WHERE thread_id = $1 ) t WHERE id = $2", 
    thread_id, msg_id)
    .fetch_one(&db_pool)
    .await.map_err(SomeSortOfError::new)?;

    let position = match row.position {
        Some(p) => p,
        None => {
            return Err(SomeSortOfError::new("Could not get position"));
        }
    };
    debug!("Message position: {position}");

    let page = (position as f64 / msgs_per_page as f64).ceil() as i64;

    Ok(page)
}
pub async fn get_message(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> Result<ForumThreadMessage, SomeSortOfError> {
    let msg = sqlx::query_as!(
        ForumThreadMessage,
        "SELECT primary_key, id, thread_id FROM forum_thread_messages WHERE id = $1",
        msg_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(msg)
}

pub async fn get_message_by_position_in_thread(
    db_pool: sqlx::PgPool,
    thread_id: String,
    position: u64,
) -> Result<ForumThreadMessage, SomeSortOfError> {
    let position = i64::try_from(position).map_err(SomeSortOfError::new)?;
    let msg = sqlx::query_as!(
        ForumThreadMessage,
        "SELECT primary_key, id, thread_id FROM forum_thread_messages WHERE thread_id = $1 ORDER BY primary_key ASC LIMIT 1 OFFSET $2",
        thread_id, position
    )
    .fetch_one(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(msg)
}
