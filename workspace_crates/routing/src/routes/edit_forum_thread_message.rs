
use abac::CanEditForumThreadMessageInput;
use error_structs::InternalAppError;
use http_stuff::http_response::{HttpResponseData, redirect_to};

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let the_user = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let msg_id = {
        let msg_id = router_data.http_request_data.query.get("message_id");
        if let Some(msg_id) = msg_id {
            msg_id.to_owned()
        } else {
            return Ok(redirect_to("/".to_string()));
        }
    };
    let msg = forum::forum_thread_message::get_message(db_pool.clone(), msg_id.clone()).await.map_err(InternalAppError::new)?;
    let original_message_poster_id = forum::get_initial_message_version(db_pool.clone(), msg_id.clone()).await.map_err(InternalAppError::new)?.user_id;
    let current_message_version = forum::get_latest_message_version(db_pool.clone(), msg_id.clone()).await.map_err(InternalAppError::new)?;
    if !abac::can_edit_forum_thread_message(CanEditForumThreadMessageInput{
        original_message_poster_id,
        user_id: the_user.id,
    }).await.map_err(InternalAppError::new)?    {
        return Ok(redirect_to("/".to_string()));
    }
    

    todo!();
}
