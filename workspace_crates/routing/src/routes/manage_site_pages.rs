use abac::CanManageSitePagesInput;
use error_structs::InternalAppError;
use html_templating::html_template_structs;
use http_stuff::http_response::{HttpResponseData, redirect_to};

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in
        || !abac::can_manage_site_pages(CanManageSitePagesInput {
            db_pool: db_pool.clone(),
            session_id: session.id.clone(),
        })
        .await
        .map_err(InternalAppError::new)?
    {
        session::add_session_notification_error(db_pool.clone(), session.id.clone(), String::from("Insufficient permissions to access this amazing page")).await.map_err(InternalAppError::new)?;
        return Ok(redirect_to("/".to_string()));
    }

    let pagination_page = 1;
    let pagination_records_per_page = 20;
    let site_pages = site_pages::list_of_site_pages(
        router_data.db_pool,
        pagination_page,
        pagination_records_per_page,
    )
    .await
    .map_err(InternalAppError::new)?;

    let html = html_templating::render(html_template_structs::ManageSitePagesTemplate {
        main_info: router_data.base_template_data,
        site_pages,
    });
    Ok(http_stuff::http_response::ok_html(html))
}
