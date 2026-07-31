use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, CustomPage};
use http_stuff::http_response::{HttpResponseData, ok_html, redirect_to};

use routing_structs::RouterData;
use session::add_session_notification_error;

pub async fn get(
    router_data: RouterData,
    path: String,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let custom_site_page =
        match custom_site_pages::find_custom_site_page(db_pool.clone(), path.clone())
            .await
            .map_err(InternalAppError::new)?
        {
            Some(csp) => csp,
            None => {
                add_session_notification_error(
                    db_pool.clone(),
                    session.id.clone(),
                    format!("Page {path} not found."),
                )
                .await;
                return Ok(redirect_to("/".to_string()));
            }
        };

    Ok(ok_html(html_templating::render(
        html_template_structs::ViewCustomPageTemplate {
            main_info: router_data.base_template_data,
            page: CustomPage {
                name: custom_site_page.name,
                content: custom_site_page.content,
            },
        },
    )))
}
