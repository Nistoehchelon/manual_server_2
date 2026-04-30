use abac::UserCanViewAppExecutableVersionInput;
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
    let user_can_manage_site_pages = abac::can_manage_site_pages(abac::CanManageSitePagesInput {
        db_pool: db_pool.clone(),
        user_id: user.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?;

    let user_can_view_app_executable_version = abac::user_can_view_app_executable_version(UserCanViewAppExecutableVersionInput {db_pool: db_pool.clone(), user_id: user.id.clone()}).await.map_err(InternalAppError::new)?;
    let app_executable_version = if user_can_view_app_executable_version {
        Some(app_executable_version::VERSION.to_string())
    } else {
        None
    };

    let html = html_templating::render(html_template_structs::DashboardTemplate {
        main_info: router_data.base_template_data,
        username: user.username.clone(),
        user_id: user.id.clone(),
        user_can_manage_site_pages,
        user_can_view_app_executable_version,
        app_executable_version,
    });
    Ok(http_stuff::http_response::ok_html(html))
}
