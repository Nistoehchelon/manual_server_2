use http_stuff::http_response::{self, HttpResponseData};

use routing_structs::RouterData;

pub fn dump_request(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let body_text = router_data.http_request_data.body_bytes.clone();
    let body_text = String::from_utf8_lossy(&body_text).into_owned();
    let request_dump = format!(
        "{:#?}\nBody as text:
{}",
        router_data.http_request_data, body_text
    )
    .into_bytes();
    let response = http_stuff::http_response::ok(
        request_dump,
        http_response::ContentType::TextPlain,
        Some(http_response::Charset::Utf8),
    );
    Ok(response)
}
