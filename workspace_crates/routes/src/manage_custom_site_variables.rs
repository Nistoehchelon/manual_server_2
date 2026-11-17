use abac::CanManageSitePagesInput;
use error_structs::InternalAppError;
use html_templating::html_template_structs;
use http_stuff::http_response::{HttpResponseData, redirect_to};
use routing_structs::RouterData;


pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in
        || !abac::can_manage_site_pages(CanManageSitePagesInput {
            db_pool: db_pool.clone(),
            user_id: {
                let u = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
                    .await
                    .map_err(InternalAppError::new)?;
                u.id
            },
        })
        .await
        .map_err(InternalAppError::new)?
    {
        session::add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            String::from("Insufficient permissions to access this amazing page"),
        )
        .await;
        return Ok(redirect_to("/".to_string()));
    }

   todo!(

   )
}
