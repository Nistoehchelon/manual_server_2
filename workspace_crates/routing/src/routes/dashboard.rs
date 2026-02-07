use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::http_response::{ResponseData, redirect_to};
use user;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        request_data: _,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let user = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let notifications =
        session::read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
            .await
            .map_err(InternalAppError::new)?;

    let html = html_templating::render(html_template_structs::DashboardTemplate {
        main_info: MainInfo {
            title: "Your Dashboard".to_string(),
            notifications: into_vec::into_vec(notifications),
        },
        username: user.username.clone(),
    });
    Ok(http_stuff::http_response::ok_html(html))
}
