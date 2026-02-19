use std::fmt::Debug;
use std::fmt::Display;
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    Code200Ok,
    Code302Found,
    Code303SeeOther,
    Code500InternalServerError,
    Code503ServiceUnavailable,
    Code400BadRequest,
    Code408RequestTimeout,
    Code429TooManyRequests,
    Code413ContentTooLarge,
    Code414UriTooLong,
    Code505HttpVersionNotSupported,
    Code431RequestHeaderFieldsTooLarge,
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Code200Ok => "200 OK",
            Self::Code302Found => "302 Found",
            Self::Code303SeeOther => "303 See Other",
            Self::Code400BadRequest => "400 Bad Request",
            Self::Code408RequestTimeout => "408 Request Timeout",
            Self::Code413ContentTooLarge => "413 Content Too Large",
            Self::Code414UriTooLong => "414 Uri Too Long",
            Self::Code429TooManyRequests => "429 Too Many Requests",
            Self::Code431RequestHeaderFieldsTooLarge => "431 Request Header Fields Too Large",
            Self::Code500InternalServerError => "500 Internal Server Error",
            Self::Code503ServiceUnavailable => "503 Service Unavailable",
            Self::Code505HttpVersionNotSupported => "505 HTTP Version Not Supported",
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Clone)]
pub struct SetCookieHeader {
    pub key: String,
    pub value: String,
    pub path: String,
}
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]

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


#[derive(Debug, Clone)]
pub struct HttpResponseData {
    pub body: Option<Vec<u8>>,
    pub headers: Headers,
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub status_code: StatusCode,
    pub content_type: ContentType,
    pub charset: Option<Charset>,
    pub set_cookie: Option<Vec<SetCookieHeader>>,
    pub location: Option<String>,
    pub connection: KeepAliveOrClose,
    pub other_headers: Option<HashMap<String, String>>,
}

pub fn ok_html(content: String) -> HttpResponseData {
    ok(
        content.into_bytes(),
        ContentType::TextHtml,
        Some(Charset::Utf8),
    )
}

pub fn ok(content: Vec<u8>, content_type: ContentType, charset: Option<Charset>) -> HttpResponseData {
    HttpResponseData {
        body: Some(content),
        headers: Headers {
            status_code: StatusCode::Code200Ok,
            content_type,
            charset,
            set_cookie: None,
            location: None,
            connection: KeepAliveOrClose::KeepAlive,
            other_headers: None,
        },
    }
}

pub fn response(
    HttpResponseData {
        body,
        headers:
            Headers {
                status_code,
                set_cookie,
                content_type,
                charset,
                location,
                connection,
                other_headers,
            },
    }: HttpResponseData,
) -> Result<HttpResponse, HttpResponseError> {
    let mut http_response_string: String = String::new();
    http_response_string.push_str("HTTP/1.1");
    http_response_string.push(' ');
    let status_code_string = status_code.to_string();
    http_response_string.push_str(&status_code_string);
    http_response_string.push_str("\r\n");

    if let Some(set_cookie) = set_cookie {
        for sc in set_cookie.iter() {
            let set_cookie_string = format!(
                "Set-Cookie: {}={}; Path={}; Secure; HttpOnly; SameSite=Strict\r\n",
                sc.key, sc.value, sc.path
            );
            http_response_string.push_str(&set_cookie_string);
        }
    }

    http_response_string.push_str(&format!("Connection: {}\r\n", connection));

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

    http_response_string.push_str("X-Content-Type-Options: nosniff\r\n");
    http_response_string.push_str("X-Frame-Options: DENY\r\n");

    http_response_string.push_str("Cross-Origin-Embedder-Policy: require-corp\r\n");
    http_response_string.push_str("Cross-Origin-Resource-Policy: same-origin\r\n");
    http_response_string.push_str("Cross-Origin-Opener-Policy: same-origin\r\n");
    http_response_string.push_str("Referrer-Policy: no-referrer\r\n");
    http_response_string.push_str(
        "Permissions-Policy:\
camera=(),\
microphone=(),\
geolocation=(),\
payment=(),\
usb=(),\
bluetooth=()\r\n",
    );

    if let Some(other_headers) = other_headers {
        for (name, value) in other_headers.iter() {
        http_response_string
        .push_str(&format!("{name}: {value}\r\n"));
    }
}


    let nonce = "test_make_this_random_later";
    http_response_string.push_str(&format!(
        "Content-Security-Policy: default-src 'none'; script-src 'nonce-{nonce}' 'strict-dynamic'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; img-src 'self'; style-src 'self'; connect-src 'self'; font-src 'self'; form-action 'self';"
    ));

    http_response_string.push_str("\r\n\r\n");

    if !http_response_string.is_ascii() {
        let err = "http_response_string contains non-ascii bytes, not allowed".to_string();
        dbg!(&err);
        return Err(HttpResponseError::new(err));
    }

    let mut http_response_bytes = http_response_string.clone().into_bytes();

    if body_bytes_length > 0 {
        http_response_bytes.extend_from_slice(&body_bytes);
    }

    Ok(http_response_bytes)
}

pub fn redirect_to(page: String) -> HttpResponseData {
    HttpResponseData {
        body: None,
        headers: Headers {
            status_code: StatusCode::Code302Found,
            content_type: ContentType::TextHtml,
            location: Some(page.to_string()),
            charset: None,

            set_cookie: None,
            connection: KeepAliveOrClose::KeepAlive,
            other_headers: None,
        },
    }
}

pub fn internal_server_error_500() -> HttpResponseData {
    HttpResponseData {
        body: None,
        headers: Headers {
            status_code: StatusCode::Code500InternalServerError,
            content_type: ContentType::TextHtml,
            charset: Some(Charset::Utf8),
            set_cookie: None,
            location: None,
            connection: KeepAliveOrClose::KeepAlive,
            other_headers: None,
        },
    }
}
pub fn bad_request_400() -> HttpResponseData {
    HttpResponseData {
        body: None,
        headers: Headers {
            status_code: StatusCode::Code400BadRequest,
            content_type: ContentType::TextHtml,
            charset: Some(Charset::Utf8),

            set_cookie: None,
            location: None,
            connection: KeepAliveOrClose::KeepAlive,
            other_headers: None,
        },
    }
}

/// I mainly use this at main.rs when if an error occurs where is nothing else to be done and the
/// more sophisticated response mechanisms above, like response() might have been a part of the error.
/// I do not actually know whether response is or not at any given case, but I do not want to have to care
pub fn standalone_response(status_code: StatusCode, msg: String) -> HttpResponse {
    let body = format!(
        r##"<!DOCTYPE html><html lang = "en"><head><title>{msg}</title><style> body {{ text-align: center; }}</style></head><body><div>{msg}</div></body></html>"##
    ).into_bytes();
    let response_data = HttpResponseData {
        body: Some(body),
        headers: Headers {
            status_code: status_code.clone(),
            content_type: ContentType::TextHtml,
            charset: Some(Charset::Utf8),
            set_cookie: None,
            location: None,
            connection: KeepAliveOrClose::Close,
            other_headers: None,
        },
    };
    response(response_data).unwrap_or_else(|_| {
        let msg = format!("Some Error occured {msg} {}", status_code);
        format!(
            "HTTP/1.1 {}\n\
Connection: Close\r\n\
Content-Type: text/plain\r\n\
Content-Length:\r\n\r\n\
{msg}",
            status_code
        )
        .into_bytes()
    })
}
