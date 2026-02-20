use std::fmt::Debug;
use std::fmt::Display;

use indexmap::IndexMap;
use indexmap::map::Iter;

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
    pub secure: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    TextHtml,
    TextCss,
    TextPlain,
    TextJavascript,
    ImageXIcon,
}
impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            Self::TextHtml => "text/html",
            Self::TextCss => "text/css",
            Self::TextPlain => "text/plain",
            Self::ImageXIcon => "image/x-icon",
            Self::TextJavascript => "text/javascript",
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
    pub status_code: StatusCode,
    pub set_cookie: Option<Vec<SetCookieHeader>>,
    pub headers: Headers,
}
impl HttpResponseData {
    pub fn response(&self,
) -> Result<HttpResponse, HttpResponseError> {
    let HttpResponseData {
        body,
        status_code,
        headers, set_cookie } = self;
    let mut http_response_string: String = String::new();
    http_response_string.push_str("HTTP/1.1");
    http_response_string.push(' ');
    let status_code_string = status_code.to_string();
    http_response_string.push_str(&status_code_string);
    http_response_string.push_str("\r\n");

    if let Some(set_cookie) = set_cookie {
        for sc in set_cookie.iter() {
            dbg!(sc.clone());
            let secure = 's: {
                if sc.secure {
                    break 's "Secure;";
                }
                ""
            }.to_string();
            let set_cookie_string = format!(
                "Set-Cookie: {}={}; Path={}; {secure} HttpOnly; SameSite=Strict\r\n",
                sc.key, sc.value, sc.path
            );
            http_response_string.push_str(&set_cookie_string);
        }
    }

    let mut body_bytes_length = 0;
    let body_bytes: Vec<u8> = match body {
        Some(b) => {
            body_bytes_length = b.len();
            b.to_vec()
        }
        None => Vec::new(),
    };

     http_response_string.push_str(&format!("Content-Length: {body_bytes_length}\r\n"));



        for (name, value) in headers.iter() {
            http_response_string.push_str(&format!("{name}: {value}\r\n"));
        }
    


    http_response_string.push_str("\r\n");

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
}

#[derive(Debug, Clone)]
pub struct Headers {
    inner: IndexMap<String, String>,
}
#[derive(Debug, Clone)]
pub enum HeaderKind {
     Singleton,
     CommaList,
     SetCookie,
}
impl Headers {
    pub fn classify(name: &str) -> HeaderKind {
        match name {
            // list headers (RFC 9110 list semantics)
            "accept" | "accept-encoding" | "accept-language" | "cache-control" | "warning" => {
                HeaderKind::CommaList
            }

            "set-cookie" => HeaderKind::SetCookie,

            _ => HeaderKind::Singleton,
        }
    }
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }
    pub fn insert(&mut self, name: &str, value: String) {
        let name = name.to_ascii_lowercase();
        let kind = Headers::classify(&name);
        match kind {
            HeaderKind::Singleton => {
                self.inner.insert(name, value.trim().to_string());
            }

            HeaderKind::CommaList => {
                self.inner
                    .entry(name)
                    .and_modify(|v| {
                        v.push_str(", ");
                        v.push_str(value.trim());
                    })
                    .or_insert_with(|| value.trim().to_string());
            }

            HeaderKind::SetCookie => {
            }
        }
    }
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.inner.shift_remove(name)
    }
    pub fn get(&self, name: &str) -> Option<String> {
        self.inner.get(name).cloned()
    }
    
    pub fn append(&mut self, name: &str, value: String) {
        let existing_val = self.get(name);

        let mut to_append = "".to_string();
        if let Some(existing_val) = existing_val {
            to_append.push_str(&existing_val);
        }
        to_append.push_str(&value);
        self.insert(name, to_append)
    }
    pub fn iter(&self) -> Iter<'_, String, String> {
        self.inner.iter()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

pub fn ok_html(content: String) -> HttpResponseData {
    ok(
        content.into_bytes(),
        ContentType::TextHtml,
        Some(Charset::Utf8),
    )
}

pub fn ok(
    content: Vec<u8>,
    content_type: ContentType,
    charset: Option<Charset>,
) -> HttpResponseData {
    HttpResponseData {
        set_cookie: None,
        body: Some(content),
        status_code: StatusCode::Code200Ok,
        headers: {
            let mut headers = Headers::new();
            headers.insert("content-type", content_type.to_string());
            if let Some(charset) = charset {
                headers.append(
                    "content-type",
                    format!("; charset={}", charset),
                );
            }
            headers
        },
    }
}

pub fn redirect_to(page: String) -> HttpResponseData {
    HttpResponseData {
        body: None,
        status_code: StatusCode::Code302Found,
        headers:  {
            let mut headers = Headers::new();
            headers.insert("content-type", ContentType::TextHtml.to_string());
            headers.insert("location", page.to_string());
            headers.insert("connection", KeepAliveOrClose::KeepAlive.to_string());
            headers
        },
        set_cookie: None,
    }
}

pub fn internal_server_error_500() -> HttpResponseData {
    HttpResponseData {
        body: None,
        status_code: StatusCode::Code500InternalServerError,
        headers:  {
            let mut headers = Headers::new();
            headers.insert("content-type", ContentType::TextHtml.to_string());
            headers.insert("charset", Charset::Utf8.to_string());
            headers.insert("connection", KeepAliveOrClose::KeepAlive.to_string());
            headers
        },
        set_cookie: None,
    }
}
pub fn bad_request_400() -> HttpResponseData {
    HttpResponseData {
        body: None,
        status_code: StatusCode::Code400BadRequest,
        headers:  {
            let mut headers = Headers::new();
            headers.insert("content-type", ContentType::TextHtml.to_string());
            headers.insert("charset", Charset::Utf8.to_string());
            headers.insert("connection", KeepAliveOrClose::KeepAlive.to_string());
            headers
        },
        set_cookie: None,
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
        status_code: status_code.clone(),
        headers: {
           let mut headers = Headers::new();
            headers.insert("content_type", ContentType::TextHtml.to_string());
            headers.insert("charset", Charset::Utf8.to_string());

            headers.insert("connection", KeepAliveOrClose::Close.to_string());
            headers
        },
        set_cookie: None,
    };
    response_data.response().unwrap_or_else(|_| {
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
