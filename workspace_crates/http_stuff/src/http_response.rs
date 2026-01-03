pub enum ResponseCode {
    Ok200,
    InternalServerError500,
    ContentTooLarge413,
}

pub struct SetCookieHeader {
    pub key: String,
    pub value: String,
}
pub struct ResponseData {
    pub status_code: String,
    pub content_type: String,
    pub body: Option<Vec<u8>>,
    pub set_cookie_headers: Option<Vec<SetCookieHeader>>,
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

pub fn response(response_data: ResponseData) -> String {
    let status_code = match code {
        ResponseCode::InternalServerError500 => format!("500 Internal Server Error"),
        ResponseCode::Ok200 => format!("200 OK"),
        ResponseCode::ContentTooLarge413 => format!("413 Content Too Large"),
    };
    let mut set_cookie_headers_string = format!("");
    match set_cookie {
        Some(set_cookie) => {
            for sc in set_cookie.iter() {
                let (key, value) = sc;
                set_cookie_headers_string = format!(
                    "{set_cookie_headers_string}Set-Cookie: {}={}; Path=/\r\n",
                    key.to_string(),
                    value.to_string()
                );
            }
        },
        None => {},
    }

    format!(
        "HTTP/1.1 {status_code}\r\n{set_cookie_headers_string}\
Connection: close\r\n\
Content-Type: {content_type}; charset=utf-8\r\n\r\n\
{content}",
    )
}

pub fn redirect_to(page: String) -> String {
    format!(
        "HTTP/1.1 302 Found\r\n\
Location: {page}\r\n\
Connection: close\r\n\
Content-Type: text/html\r\n\r\n"
    )
}
