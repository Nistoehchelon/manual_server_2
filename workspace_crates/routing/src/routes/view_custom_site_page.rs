use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, CustomPage};
use http_stuff::http_response::{HttpResponseData, ok_html, redirect_to};

use crate::RouterData;

pub async fn get(
    router_data: RouterData, path: String
) -> Result<HttpResponseData, error_structs::InternalAppError> {
let db_pool = router_data.db_pool.clone();
let session = router_data.session.clone();
 let thread_id = path.clone();
let thread = match forum::get_thread(db_pool.clone(), thread_id.clone()).await {
  Ok(t) => t,
  Err(_) => {  session::add_session_notification_error(db_pool.clone(), session.id.clone(), format!("Could not find the requested page `{path}`")).await;
    return Ok(redirect_to("/".to_string()));}
};
let content: String = {
    let thread_page = 1;
    let limit = 1;
    let msg_id = forum::get_thread_message_ids(db_pool.clone(), thread_id.clone(), thread_page, limit).await.map_err(InternalAppError::new)?;
    let msg_id = msg_id[0].to_string();
    let msg = forum::get_latest_message_version(db_pool.clone(), msg_id.clone()).await.map_err(InternalAppError::new)?;
    let msg_content = msg.content;
    custom_site_pages::replace_variables_with_contents(db_pool.clone(), msg_content.to_string()).await.map_err(InternalAppError::new)?

};

 Ok(ok_html(html_templating::render(html_template_structs::ViewCustomPageTemplate {
        main_info: router_data.base_template_data,
        page: CustomPage { name: thread.name, content },
    })))

}