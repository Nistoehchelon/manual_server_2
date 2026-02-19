use error_structs::InternalAppError;
use http_stuff::http_response::{HttpResponseData, redirect_to};
use session::edit_session;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        request_data: _,
    }: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let mut clone_sess = session.clone();
    clone_sess.logged_in = false;
    clone_sess.user_id = "".to_string();
    edit_session(db_pool, clone_sess.clone().id, clone_sess.clone())
        .await
        .map_err(InternalAppError::new)?;

    Ok(redirect_to("/".to_string()))
}
