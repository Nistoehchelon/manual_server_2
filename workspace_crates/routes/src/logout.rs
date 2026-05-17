use error_structs::InternalAppError;
use http_stuff::http_response::{HttpResponseData, redirect_to};
use session::edit_session;

use routing_structs::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let session = router_data.session.clone();
    let db_pool = router_data.db_pool.clone();
    let mut clone_sess = session.clone();
    clone_sess.logged_in = false;
    clone_sess.user_id = "".to_string();
    edit_session(db_pool, clone_sess.clone().id, clone_sess.clone())
        .await
        .map_err(InternalAppError::new)?;

    Ok(redirect_to("/".to_string()))
}
