use encoding::decode_request_string;
use indexmap::IndexMap;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use url::Query;

pub const MAX_HEADER_BYTES: usize = 8 * 1024; // 8 KB
pub const MAX_HEADERS_COUNT: usize = 50; // 8 KB

pub const MAX_HEADER_NAME_LEN: usize = 100;
pub const MAX_HEADER_VALUE_LEN: usize = 2048;

pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB

pub const STREAM_READ_TIMEOUT_SECS: u64 = 3; // TO USE
pub const STREAM_WRITE_TIMEOUT_SECS: u64 = 3; // TO USE

pub const MAX_REQUEST_TARGET_LENGTH: usize = 150;

pub const MAX_COOKIES_PER_REQUEST: usize = 50;

/// Holds all the parsed information from an HTTP request.
#[derive(Clone, Debug)]
pub struct HttpRequestData {
    pub headers: Headers,
    pub method: String,
    pub path: String,
    pub query: Query,
    pub path_and_query: String,
    pub body_bytes: Vec<u8>,
    pub cookies: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum HttpRequestParsingError {
    TooManyCookies,
    HttpVersionNotSupported,
    UriTooLong,
    SomeSortOfStreamError,
    ContentTooLarge,
    BadRequest,
    UnexpectedEof,
    TooManyHeaders,
    HeaderNameTooLong,
    HeaderValueTooLong,
}
impl Display for HttpRequestParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BadRequest => "Bad Request",
            Self::ContentTooLarge => "Content Too Large",
            Self::SomeSortOfStreamError => "Some Sort Of Stream Error",
            Self::UnexpectedEof => "Unexpected End Of File",
            Self::UriTooLong => "Uri Too Long",
            Self::HttpVersionNotSupported => "HTTP Version Not Supported",
            Self::TooManyHeaders => "Too Many Headers",
            Self::TooManyCookies => "Too Many Cookies",
            Self::HeaderNameTooLong => "Header Name Too Long",
            Self::HeaderValueTooLong => "Header Value Too Long",
        };
        write!(f, "{}", s)
    }
}
pub struct ParsedHeaderStream {
    pub headers: Vec<u8>,
    pub leftover: Vec<u8>,
}

pub fn parse_body_into_string_form_input_params(
    body: Vec<u8>,
) -> Result<HashMap<String, String>, String> {
    let body = match String::from_utf8(body) {
        Ok(b) => b,
        Err(e) => return Err(format!("Could not parse body into valid utf8, {e:#?}")),
    };
    dbg!("body raw unparsed string", body.clone());

    let body_clone = body.clone();
    let split: Vec<&str> = body_clone.split('&').collect();
    let mut params: HashMap<String, String> = HashMap::new();
    for pair in split {
        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => {
                dbg!("Could not split post param");
                return Err(format!("Could not split by = in post"));
            }
        };

        let key = match decode_request_string(key.to_string()) {
            Err(e) => {
                return Err(format!("Could not decode key into string, {e:#?}"));
            }
            Ok(s) => s,
        };

        let value = match decode_request_string(value.to_string()) {
            Err(e) => {
                return Err(format!("Could not decode key into string, {e:#?}"));
            }
            Ok(s) => s,
        };

        params.insert(key.to_string(), value.to_string());
    }
    return Ok(params);
}

async fn parse_headers_stream(
    stream: &mut tokio::net::TcpStream,
) -> Result<ParsedHeaderStream, HttpRequestParsingError> {
    const BYTES_PER_READ: usize = 1024;

    let mut byte_buf = [0u8; BYTES_PER_READ];
    let mut header_bytes = Vec::new();
    let mut read_total_bytes = 0;
    let mut count_calls_to_stream_read = 0;

    loop {
        count_calls_to_stream_read += 1;
        let bytes_read = match stream.read(&mut byte_buf).await {
            Ok(0) => {
                dbg!(format!(
                    "Connection closed?  before headers completed. 0 bytes read this stream.read call  (in total read {read_total_bytes} bytes, having called stream.read {count_calls_to_stream_read} times) "
                ));
                return Err(HttpRequestParsingError::BadRequest);
            }

            Ok(n) => n,
            Err(e) => {
                dbg!("Stream read error", e);
                dbg!(format!(
                    "(in total read {read_total_bytes} having called stream.read {count_calls_to_stream_read} times) "
                ));
                return Err(HttpRequestParsingError::SomeSortOfStreamError);
            }
        };

        read_total_bytes += bytes_read;

        if read_total_bytes > MAX_HEADER_BYTES {
            dbg!(
                "Header size exceeded max allowed",
                read_total_bytes,
                MAX_HEADER_BYTES
            );
            return Err(HttpRequestParsingError::ContentTooLarge);
        }

        // Append the bytes we just read
        header_bytes.extend_from_slice(&byte_buf[..bytes_read]);

        // Check for header-body boundary
        if let Some(pos) = header_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            // `pos` points to the start of the \r\n\r\n sequence
            let body_start = pos + 4; // skip the \r\n\r\n
            let leftover_body = header_bytes[body_start..].to_vec(); // save bytes beyond headers

            // truncate header_bytes to only the headers
            header_bytes.truncate(body_start);

            // not sure if there is a point to this check... Vec u8 will by definition only contain ascii  bytes, right?
            if !header_bytes.is_ascii() {
                dbg!("I do not accept headers with non-ascii bytes");
                return Err(HttpRequestParsingError::BadRequest);
            }

            return Ok(ParsedHeaderStream {
                headers: header_bytes,
                leftover: leftover_body,
            });
        }
    }
}

pub fn count_occurences_of_rn(s: &str) -> usize {
    s.match_indices("\r\n").count()
}

pub fn all_characters_in_header_name_are_valid(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            matches!(b,
                b'A'..=b'Z' |
                b'a'..=b'z' |
                b'0'..=b'9' |
                b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' |
                b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
        })
}
pub fn request_target_characters_are_valid(path: &str) -> bool {
    path.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                '-' | '.'
                    | '_'
                    | '~'
                    | '!'
                    | '$'
                    | '&'
                    | '\''
                    | '('
                    | ')'
                    | '*'
                    | '+'
                    | ','
                    | ';'
                    | '='
                    | ':'
                    | '@'
                    | '/'
                    | '?'
                    | '%'
            )
    })
}

pub async fn parse_http_request(
    stream: &mut tokio::net::TcpStream,
) -> Result<HttpRequestData, HttpRequestParsingError> {
    let ParsedHeaderStream {
        headers,
        leftover: leftover_from_headers_parsing,
    } = parse_headers_stream(stream).await?;
    let headers_string = String::from_utf8_lossy(&headers).to_string();
    let headers_count = count_occurences_of_rn(&headers_string);

    if headers_count > MAX_HEADERS_COUNT {
        return Err(HttpRequestParsingError::TooManyHeaders);
    }

    let mut headers: Headers = Headers::default();
    let mut header_string_lines = headers_string.split("\r\n");
    let first_line = header_string_lines
        .next()
        .ok_or(HttpRequestParsingError::BadRequest)?;
    for line in header_string_lines {
        if line.is_empty() {
            break;
        }

        if line.contains('\n') || line.contains('\r') {
            return Err(HttpRequestParsingError::BadRequest);
        }
        let (key, value) = match line.split_once(':') {
            Some(parts) => parts,
            None => {
                dbg!("Header has no semicolon, invalid");
                return Err(HttpRequestParsingError::BadRequest);
            }
        };
        if key.len() > MAX_HEADER_NAME_LEN {
            return Err(HttpRequestParsingError::HeaderNameTooLong);
        }
        if value.len() > MAX_HEADER_VALUE_LEN {
            return Err(HttpRequestParsingError::HeaderValueTooLong);
        }
        if !all_characters_in_header_name_are_valid(key) {
            dbg!("Header name ", key, "contains invalid chars");
            return Err(HttpRequestParsingError::BadRequest);
        }
        headers
            .insert(key.to_lowercase().as_str(), value.trim())
            .map_err(|e| {
                dbg!(e);
                HttpRequestParsingError::BadRequest
            })?;
    }
    if !(headers.contains("host")) {
        return Err(HttpRequestParsingError::BadRequest);
    }

    let first_line_split = first_line.split(" ").collect::<Vec<&str>>();
    if first_line_split.len() != 3 {
        dbg!(format!(
            "I expected something other than {first_line_split:#?} for first header line"
        ));
        return Err(HttpRequestParsingError::BadRequest);
    }
    let method = first_line_split[0].to_string();
    let path_and_query = first_line_split[1].to_string();
    let http_version = first_line_split[2].to_string();

    if http_version != "HTTP/1.1" {
        return Err(HttpRequestParsingError::HttpVersionNotSupported);
    }

    if path_and_query.len() > MAX_REQUEST_TARGET_LENGTH {
        return Err(HttpRequestParsingError::UriTooLong);
    }
    if !request_target_characters_are_valid(&path_and_query) {
        return Err(HttpRequestParsingError::BadRequest);
    }

    let (path, query) = parse_path_and_query(path_and_query.clone());

    // --- Parse cookies ---
    let cookies = parse_cookies(headers.clone())?;

    let mut body: Vec<u8> = vec![];

    let mut content_length = match headers
        .get("content-length")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()
    {
        Err(e) => {
            dbg!("Parse error", e);
            return Err(HttpRequestParsingError::BadRequest);
        }
        Ok(v) => v,
    };
    if content_length > MAX_BODY_BYTES {
        dbg!(content_length, "exceeds", MAX_BODY_BYTES);
        return Err(HttpRequestParsingError::ContentTooLarge);
    } else if method == "POST" {
        if leftover_from_headers_parsing.len() > 0 {
            if leftover_from_headers_parsing.len() > content_length {
                dbg!(
                    "Leftover from header reading is greater than promised content length, rejecting",
                    leftover_from_headers_parsing.len(),
                    content_length
                );
                return Err(HttpRequestParsingError::BadRequest);
            }
            content_length -= leftover_from_headers_parsing.len();
            body.extend(leftover_from_headers_parsing);
        }
        if content_length > 0 {
            let mut reader = BufReader::new(stream);
            let mut body_bytes_exact = vec![0; content_length];
            match reader.read_exact(&mut body_bytes_exact).await {
                Err(e) => {
                    dbg!("read_exact error body", e);
                    return Err(HttpRequestParsingError::BadRequest);
                }
                Ok(_) => {}
            }
            body.extend(body_bytes_exact);
        }
    }

    Ok(HttpRequestData {
        method: method,
        path,
        path_and_query: path_and_query,
        query,
        body_bytes: body,
        cookies,
        headers,
    })
}

fn parse_path_and_query(path_with_query: String) -> (String, Query) {
    let parts: Vec<&str> = path_with_query.split('?').collect();
    let path = parts[0].to_string();
    let mut query = IndexMap::new();

    if parts.len() > 1 {
        for pair in parts[1].split('&') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                query.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    (path, query)
}

fn parse_cookies(headers: Headers) -> Result<HashMap<String, String>, HttpRequestParsingError> {
    if let Some(cookie_header) = headers.get("cookie") {
        let cookies_count = cookie_header.matches(";").count();
        if cookies_count > MAX_COOKIES_PER_REQUEST {
            return Err(HttpRequestParsingError::TooManyCookies);
        }

        let mut cookies = HashMap::new();
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.trim().split('=').collect();
            if parts.len() == 2 {
                cookies.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
        return Ok(cookies);
    }
    Ok(HashMap::new())
}

pub trait StripPrefixASCIICaseInsensitive {
    fn strip_prefix_ascii_case_insensitive(&self, prefix: &str) -> Option<&str>;
}
impl StripPrefixASCIICaseInsensitive for str {
    fn strip_prefix_ascii_case_insensitive(&self, prefix: &str) -> Option<&str> {
        if self.len() >= prefix.len() && self[..prefix.len()].eq_ignore_ascii_case(prefix) {
            Some(&self[prefix.len()..])
        } else {
            None
        }
    }
}

/// Wrapper for HTTP headers
#[derive(Debug, Default, Clone)]
pub struct Headers {
    inner: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum HeaderKind {
    Singleton,
    CommaList,
    Cookie,
}

fn classify(name: &str) -> HeaderKind {
    match name {
        // list headers (RFC 9110 list semantics)
        "accept" | "accept-encoding" | "accept-language" | "cache-control" | "warning" => {
            HeaderKind::CommaList
        }

        "cookie" => HeaderKind::Cookie,

        _ => HeaderKind::Singleton,
    }
}

fn validate_cookie_duplicates(cookie_header: &str) -> Result<(), &'static str> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();

    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some((name, _)) = pair.split_once('=') {
            let name = name.trim();
            if !seen.insert(name) {
                return Err("Duplicate cookie name");
            }
        }
    }
    Ok(())
}

impl Headers {
    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), &'static str> {
        let name = name.to_ascii_lowercase();
        let kind = classify(&name);

        match kind {
            HeaderKind::Singleton => {
                if self.inner.contains_key(&name) {
                    return Err("Duplicate singleton header");
                }
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

            HeaderKind::Cookie => {
                self.inner
                    .entry(name)
                    .and_modify(|existing| {
                        existing.push_str("; ");
                        existing.push_str(value.trim());
                    })
                    .or_insert_with(|| value.trim().to_string());

                // After merge, validate cookie keys
                if let Some(cookies) = self.inner.get("cookie") {
                    validate_cookie_duplicates(cookies)?;
                }
            }
        }

        Ok(())
    }

    /// Get the first value of a header
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner
            .get(&name.to_ascii_lowercase())
            .map(|s| s.as_str())
    }

    /// Check if a header exists
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_ascii_lowercase())
    }
}
