use error_structs::InternalAppError;
use html_templating::html_template_structs::{self};
use http_stuff::http_response::{HttpResponseData, redirect_to};
use user;

use crate::RouterData;

pub async fn get(
    mut router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let user = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;

    router_data.base_template_data.title = "Your Dashboard".to_string();
    router_data.base_template_data.notifications = into_vec::into_vec(router_data.session_notifications);
    let user_can_manage_site_pages = abac::can_manage_site_pages(abac::CanManageSitePagesInput {
        db_pool,
        session_id: session.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?;
    let html = html_templating::render(html_template_structs::DashboardTemplate {
        main_info: router_data.base_template_data,
        username: user.username.clone(),
        user_can_manage_site_pages,
    });
    Ok(http_stuff::http_response::ok_html(html))
}
