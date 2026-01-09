use db_manager::{DbError, sql_queries};
use session_structs_and_stuff::Session;

pub async fn can_create_forum(db_pool: &sqlx::PgPool, session: Session) -> Result<bool, DbError> {
    let user_id = session.user_id;
   sql_queries::user_has_create_forum_permission(db_pool, user_id).await 
}