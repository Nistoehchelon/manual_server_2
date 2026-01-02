pub enum ResponseCode {
    Ok200,
    InternalServerError500,
}

pub fn ok_html(content: String) -> String {
    ok(content, "text/html".to_string())
}

pub fn ok(content: String, content_type: String) -> String {
    response(ResponseCode::Ok200, content_type, content)
}
pub fn internal_server_error_500() -> String {
    response(
        ResponseCode::InternalServerError500,
        "text/plain".to_string(),
        "Internal Server Error".to_string(),
    )
}

pub fn response(code: ResponseCode, content_type: String, content: String) -> String {
    let status_code = match code {
        ResponseCode::InternalServerError500 => format!("500 Internal Server Error"),
        ResponseCode::Ok200 => format!("200 OK"),
    };

    format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: {}; charset=utf-8\r\n\r\n\
            {}",
        status_code, content_type, content
    )
}


pub fn redirect_to(page: String) -> String {
    format!(
        "HTTP/1.1 302 Found\r\n\
         Location: {page}\r\n\
         Content-Type: text/html\r\n\r\n"
    )
}
