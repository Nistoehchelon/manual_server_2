
pub type HttpResponse = Vec<u8>;
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
    pub code: ResponseCode,
    pub content_type: String,
    pub body: Option<Vec<u8>>,
    pub set_cookie_headers: Option<Vec<SetCookieHeader>>,
}

pub fn ok_html(content: String) -> HttpResponse {
    ok(content.into_bytes(), "text/html".to_string())
}

pub fn ok(content: Vec<u8>, content_type: String) -> HttpResponse {
    response(ResponseData { code: ResponseCode::Ok200, content_type, body: Some(content), set_cookie_headers: None} )
}
pub fn internal_server_error_500() -> HttpResponse {
    let resp = response(ResponseData { code: ResponseCode::InternalServerError500, content_type: String::from("text/plain"), body: None, set_cookie_headers: None });
    return resp;
}

pub fn response(ResponseData { code: status_code, set_cookie_headers, body, content_type }: ResponseData) -> HttpResponse {
    let status_code = match status_code {
        ResponseCode::InternalServerError500 => format!("500 Internal Server Error"),
        ResponseCode::Ok200 => format!("200 OK"),
        ResponseCode::ContentTooLarge413 => format!("413 Content Too Large"),
    };
    let mut set_cookie_headers_string = format!("");
    match set_cookie_headers {
        Some(set_cookie_headers_vec) => {
            for sc in set_cookie_headers_vec.iter() {
                let SetCookieHeader { key, value } = sc;
                set_cookie_headers_string = format!(
                    "{set_cookie_headers_string}Set-Cookie: {}={}; Path=/\r\n",
                    key.to_string(),
                    value.to_string()
                );
            }
        },
        None => {},
    }
    let mut body_bytes_length = 0;
    let body_bytes: Vec<u8> = match body {
        Some(b) =>  {
            body_bytes_length = b.len();
            b
        },
        None => Vec::new(),
    };


    let mut resp = format!(
        "HTTP/1.1 {status_code}\r\n{set_cookie_headers_string}Connection: close\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {body_bytes_length}\r\n\r\n",
    ).into_bytes();
    resp.extend_from_slice(&body_bytes);
    return resp;

}

pub fn redirect_to(page: String) -> HttpResponse {
    format!(
        "HTTP/1.1 302 Found\r\n\
Location: {page}\r\n\
Connection: close\r\n\
Content-Type: text/html\r\n\r\n"
    ).into_bytes()
}
