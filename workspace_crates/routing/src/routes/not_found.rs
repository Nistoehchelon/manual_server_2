use html_templating::{html_template_structs::NotFoundTemplate, render};
use http_stuff::http_response::{ HttpResponseData, ok_html};

use crate::RouterData;

pub fn not_found(router_data: RouterData, method: String, path: String) -> HttpResponseData {
    let tpl = NotFoundTemplate { main_info: router_data.base_template_data, path, method };
    let mut resp = ok_html(render(tpl));
    resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
    resp
}