use error_structs::SomeSortOfError;

use crate::ForumThreadMessage;

pub async fn edit_forum_thread_message(msg_id: String, new_content: String, user_id: String) -> Result<(),SomeSortOfError> {
    
    todo!();
}
pub async fn get_message(db_pool: sqlx::PgPool, msg_id: String) -> Result<ForumThreadMessage, SomeSortOfError> {
    todo!();
}