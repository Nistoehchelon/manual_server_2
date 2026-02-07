use std::fmt::Debug;
use std::fmt::Display;

pub type HttpResponse = Vec<u8>;

#[derive(Debug)]
pub struct HttpResponseError {
    pub message: String,
}
impl HttpResponseError {
    pub fn new<E: Debug>(s: E) -> Self {
        let message = format!("{s:?}");
        println!("HttpResponseError: {}", message.clone());
        Self { message }
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok200,
    Found302,
    InternalServerError500,
    ServiceUnavailable503,
    BadRequest400,
    TooManyRequests429,
    ContentTooLarge413,
    UriTooLong414,
    HttpVersionNotSupported505,
    RequestHeaderFieldsTooLarge431,
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::InternalServerError500 => "500 Internal Server Error",
            Self::ServiceUnavailable503 => "503 Service Unavailable",
            Self::Ok200 => "200 OK",
            Self::Found302 => "302 Found",
            Self::ContentTooLarge413 => "413 Content Too Large",
            Self::BadRequest400 => "400 Bad Request",
            Self::TooManyRequests429 => "429 Too Many Requests",
            Self::UriTooLong414 => "414 Uri Too Long",
            Self::HttpVersionNotSupported505 => "505 HTTP Version Not Supported",
            Self::RequestHeaderFieldsTooLarge431 => "431 Request Header Fields Too Large",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug)]
pub struct SetCookieHeader {
    pub key: String,
    pub value: String,
    pub path: String,
}
#[derive(Debug, PartialEq)]
pub enum ContentType {
    TextHtml,
    TextCss,
    TextPlain,
    ImageXIcon,
}
impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            Self::TextHtml => "text/html",
            Self::TextCss => "text/css",
            Self::TextPlain => "text/plain",
            Self::ImageXIcon => "image/x-icon",
        };
        write!(f, "{}", t)
    }
}

#[derive(Debug)]
pub enum Charset {
    Utf8,
}

impl Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Utf8 => "utf-8",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, PartialEq)]

pub enum KeepAliveOrClose {
    KeepAlive,
    Close,
}

impl Display for KeepAliveOrClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeepAliveOrClose::KeepAlive => write!(f, "keep-alive"),
            KeepAliveOrClose::Close => write!(f, "close"),
        }
    }
}
#[derive(Debug)]
pub struct ResponseData {
    pub status_code: StatusCode,
    pub content_type: ContentType,
    pub charset: Option<Charset>,
    pub body: Option<Vec<u8>>,
    pub set_cookie_headers: Option<Vec<SetCookieHeader>>,
    pub location: Option<String>,
    pub connection: KeepAliveOrClose,
}

pub fn ok_html(content: String) -> ResponseData {
    ok(
        content.into_bytes(),
        ContentType::TextHtml,
        Some(Charset::Utf8),
    )
}

pub fn ok(content: Vec<u8>, content_type: ContentType, charset: Option<Charset>) -> ResponseData {
    ResponseData {
        status_code: StatusCode::Ok200,
        content_type,
        body: Some(content),
        charset,
        set_cookie_headers: None,
        location: None,
        connection: KeepAliveOrClose::KeepAlive,
    }
}

pub fn response(
    ResponseData {
        status_code,
        set_cookie_headers,
        body,
        content_type,
        charset,
        location,
        connection,
    }: ResponseData,
) -> Result<HttpResponse, HttpResponseError> {
    let mut http_response_string: String = String::new();
    http_response_string.push_str("HTTP/1.1");
    http_response_string.push_str(" ");
    let status_code_string = status_code.to_string();
    http_response_string.push_str(&status_code_string);
    http_response_string.push_str("\r\n");

    if let Some(set_cookie_headers) = set_cookie_headers {
        for sc in set_cookie_headers.iter() {
            let set_cookie_headers_string = format!(
                "Set-Cookie: {}={}; Path={}; Secure; HttpOnly; SameSite=Strict\r\n",
                sc.key.to_string(),
                sc.value.to_string(),
                sc.path.to_string()
            );
            http_response_string.push_str(&set_cookie_headers_string);
        }
    }

    http_response_string.push_str(&format!("Connection: {}\r\n", connection.to_string()));

    let content_type_string = content_type.to_string();
    http_response_string.push_str(&format!("Content-Type: {content_type_string}"));
    if let Some(cs) = charset {
        http_response_string.push_str("; charset=");
        http_response_string.push_str(&(cs.to_string()));
    }
    http_response_string.push_str("\r\n");

    if let Some(l) = location {
        http_response_string.push_str(&format!("Location: {l}\r\n",));
    }

    let mut body_bytes_length = 0;
    let body_bytes: Vec<u8> = match body {
        Some(b) => {
            body_bytes_length = b.len();
            b
        }
        None => Vec::new(),
    };

    http_response_string.push_str(&format!("Content-Length: {body_bytes_length}\r\n"));

    http_response_string.push_str(&format!(
        "Strict-Transport-Security: max-age=999999999; includeSubDomains; preload\r\n"
    ));
    http_response_string.push_str(&format!(
        "X-Content-Type-Options: nosniff\r\n"
    ));
    http_response_string.push_str(&format!(
        "X-Frame-Options: DENY\r\n"
    ));    

    http_response_string.push_str(&format!(
        "Cross-Origin-Embedder-Policy: require-corp\r\n"
    ));
    http_response_string.push_str(&format!(
        "Cross-Origin-Resource-Policy: same-origin\r\n"
    ));
    http_response_string.push_str(&format!(
        "Cross-Origin-Opener-Policy: same-origin\r\n"
    ));
    http_response_string.push_str(&format!(
        "Cross-Origin-Opener-Policy: same-origin\r\n"
    ));
    http_response_string.push_str(&format!(
        "Referrer-Policy: no-referrer\r\n"
    ));
    http_response_string.push_str(&format!(
        "Permissions-Policy:\
camera=(),\
microphone=(),\
geolocation=(),\
payment=(),\
usb=(),\
bluetooth=()\r\n"
    ));

    let nonce = "test_make_this_random_later";
    http_response_string.push_str(&format!(
        "Content-Security-Policy: default-src 'none'; script-src 'nonce-{nonce}' 'strict-dynamic'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; img-src 'self'; style-src 'self'; connect-src 'self'; font-src 'self'; form-action 'self';"
    ));


    http_response_string.push_str("\r\n\r\n");

    if !http_response_string.is_ascii() {
        let err = format!("http_response_string contains non-ascii bytes, not allowed");
        dbg!(&err);
        return Err(HttpResponseError::new(err));
    }

    let mut http_response_bytes = http_response_string.clone().into_bytes();

    if body_bytes_length > 0 {
        http_response_bytes.extend_from_slice(&body_bytes);
    }

    return Ok(http_response_bytes);
}

pub fn redirect_to(page: String) -> ResponseData {
    ResponseData {
        status_code: StatusCode::Found302,
        content_type: ContentType::TextHtml,
        location: Some(format!("{page}")),
        charset: None,
        body: None,
        set_cookie_headers: None,
        connection: KeepAliveOrClose::Close,
    }
}

pub fn internal_server_error_500() -> ResponseData {
    let resp = ResponseData {
        status_code: StatusCode::InternalServerError500,
        content_type: ContentType::TextHtml,
        charset: Some(Charset::Utf8),
        body: None,
        set_cookie_headers: None,
        location: None,
        connection: KeepAliveOrClose::KeepAlive,
    };
    return resp;
}
pub fn bad_request_400() -> ResponseData {
    let resp = ResponseData {
        status_code: StatusCode::BadRequest400,
        content_type: ContentType::TextHtml,
        charset: Some(Charset::Utf8),
        body: None,
        set_cookie_headers: None,
        location: None,
        connection: KeepAliveOrClose::KeepAlive,
    };
    return resp;
}

/// I mainly use this at main.rs when if an error occurs where is nothing else to be done and the
/// more sophisticated response mechanisms above, like response() might have been a part of the error.
/// I do not actually know whether response is or not at any given case, but I do not want to have to care
pub fn standalone_response(status_code: StatusCode, msg: String) -> HttpResponse {
    let status_code_string = status_code.to_string();
    let body_string = format!(
        r##"<!DOCTYPE html><html lang = "en"><head><title>{msg}</title><style> body {{ text-align: center; }}</style></head><body><div>{msg}</div></body></html>"##
    );
    let body_bytes = body_string.into_bytes();
    let body_bytes_length = body_bytes.len();
    let resp = format!(
        "HTTP/1.1 {status_code_string}\r\n\
Content-Type: text/html; charset=utf-8\r\n\
Connection: close\r\n\
Content-Length: {body_bytes_length}\r\n\r\n
"
    );
    let mut resp = resp.into_bytes();
    resp.extend_from_slice(&body_bytes);
    return resp;
}
