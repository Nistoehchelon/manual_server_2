
use error_structs::InternalAppError;
use http_stuff::http_response::HttpResponseData;

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let _user = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;

    todo!();
}
