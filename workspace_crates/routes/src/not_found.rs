use error_structs::InternalAppError;
use html_templating::{html_template_structs::NotFoundTemplate, render};
use http_stuff::http_response::{HttpResponseData, ok_html};

use routing_structs::RouterData;
use tracing::debug;

pub fn not_found(router_data: RouterData) -> Result<HttpResponseData, InternalAppError> {
    debug!(
        "User tried accessing page {}, but it is not found",
        router_data.http_request_data.path.clone()
    );
    let tpl = NotFoundTemplate {
        main_info: router_data.base_template_data,
        path: router_data.http_request_data.path,
        method: router_data.http_request_data.method.to_string(),
    };
    let mut resp = ok_html(render(tpl));
    resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
    Ok(resp)
}
