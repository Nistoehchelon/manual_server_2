use std::{collections::HashMap, fmt::{self, Display}, io::{BufRead, BufReader, Read}, net::TcpStream, time::Duration};

pub const MAX_HEADER_BYTES: usize = 8 * 1024; // 8 KB
pub const MAX_HEADER_LINE_LEN: usize = 4 * 1024; // 8 KB
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
pub const STREAM_READ_TIMEOUT_SECS: u64 = 5;

pub type Query = HashMap<String, String>;

/// Holds all the parsed information from an HTTP request.
#[derive(Clone, Debug)]
pub struct HttpRequestData {
    pub headers: HashMap<String, String>,
    pub method: String,
    pub path: String,
    pub query: Query,
    pub path_with_query: String,
    pub body_bytes: Vec<u8>,
    pub cookies: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum HttpRequestParsingError {
    SomeSortOfStreamError,
    ContentTooLarge,
    BadRequest,
}
impl Display for HttpRequestParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BadRequest => "Bad Request",
            Self::ContentTooLarge => "Content Too Large",
            Self::SomeSortOfStreamError => "Some Sort Of Stream Error",
        };
        write!(f, "{}", s)
    }
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

pub fn parse_http_request(stream: &mut TcpStream) -> Result<HttpRequestData, HttpRequestParsingError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(STREAM_READ_TIMEOUT_SECS)))
        .map_err(|_| HttpRequestParsingError::SomeSortOfStreamError)?;

    let mut reader = BufReader::new(stream);

    // --- Read headers ---
    let mut headers = HashMap::new();
    let mut request_line = String::new();
    let mut total_header_bytes = 0;
    let mut at_request_line = true;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(|_| HttpRequestParsingError::SomeSortOfStreamError)?;
        if n == 0 {
            return Err(HttpRequestParsingError::BadRequest); // client closed connection
        }

        total_header_bytes += n;
        if total_header_bytes > MAX_HEADER_BYTES {
            dbg!("Headers: content too large", total_header_bytes.clone(), MAX_HEADER_BYTES);
            return Err(HttpRequestParsingError::ContentTooLarge);
        }

        if line.len() > MAX_HEADER_LINE_LEN {
            dbg!("Headers: line too large", line.clone().len(), MAX_HEADER_LINE_LEN);
            return Err(HttpRequestParsingError::ContentTooLarge);
        }

        let line = line.trim_end_matches(|c| c == '\r' || c == '\n');

        if line.is_empty() {
            break; // headers finished
        }

        if at_request_line {
            request_line = line.to_string();
            at_request_line = false;
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // --- Parse request line ---
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(HttpRequestParsingError::BadRequest);
    }

    let method = parts[0].to_string();
    let path_with_query = parts[1];
    let (path, query) = parse_path_and_query(path_with_query);

    // --- Parse body ---
    let content_length = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length > MAX_BODY_BYTES {
        dbg!("body: content too large", content_length.clone(), MAX_BODY_BYTES);
        return Err(HttpRequestParsingError::ContentTooLarge);
    }

    let mut body_bytes = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body_bytes)
            .map_err(|_| HttpRequestParsingError::SomeSortOfStreamError)?;
    }

    // --- Parse cookies ---
    let cookies = parse_cookies(headers.clone());

    Ok(HttpRequestData {
        method,
        path,
        path_with_query: path_with_query.to_string(),
        query,
        body_bytes,
        cookies,
        headers,
    })
}



fn parse_path_and_query(path_with_query: &str) -> (String, HashMap<String, String>) {
    let parts: Vec<&str> = path_with_query.split('?').collect();
    let path = parts[0].to_string();
    let mut query = HashMap::new();

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

fn parse_cookies(headers: HashMap<String, String>) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    if let Some(cookie_header) = headers.get("cookie") {
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.trim().split('=').collect();
            if parts.len() == 2 {
                cookies.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }
    cookies
}

pub fn parse_username_and_password_from_body(body: Vec<u8>) -> Result<(String, String), String> {
    let body =  String::from_utf8_lossy(&body);
    let body = match urlencoding::decode(&body) {
        Err(_) => {
            return Err(format!("Could not decode body into string"));
        },
        Ok(s) => s,
    };
    let body = body.replace("+", " ");

    // Parse username from form data
    let mut valid_body = true;
    let mut username = String::new();
    let mut password = String::new();
    let binding = body.clone();
    let split = binding.split('&');
    let binding = body.clone();
    let split_len: Vec<&str> = binding.split('&').collect();
    if split_len.len() != 2 {
        valid_body = false;
    }
    for pair in split {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() != 2 {
            valid_body = false;
        }
        if kv[0] == "username" {
            username = kv[1].to_string();
        } else if kv[0] == "password" {
            password = kv[1].to_string();
        }
    }
    if valid_body {
        Ok((username, password))
    } else {
        Err(String::from("Invalid body"))
    }
}
