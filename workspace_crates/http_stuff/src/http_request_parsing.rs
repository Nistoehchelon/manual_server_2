use std::{
    collections::HashMap,
    io::{Read},
    net::TcpStream,
    time::Duration,
};


pub const MAX_HEADER_BYTES: usize = 8 * 1024; // 8 KB
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
pub const STREAM_READ_TIMEOUT_SECS: u64 = 2;

pub type Query = HashMap<String, String>;

/// Holds all the parsed information from an HTTP request.
 
pub struct ParsedHttpRequestStream {
    request_line_and_headers: String,
    body_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct HttpRequestData {
    pub method: String,
    pub path: String,
    pub query: Query,
    pub path_with_query: String,
    pub body_bytes: Vec<u8>,
    pub cookies: HashMap<String, String>,
}

pub enum HttpRequestParsingError {
    SomeSortOfStreamError,
    ContentTooLarge,
    BadRequest
}
pub trait StripPrefixASCIICaseInsensitive {
  fn strip_prefix_ascii_case_insensitive(&self, prefix: &str) -> Option<&str> ;
}
impl StripPrefixASCIICaseInsensitive for str {
    fn strip_prefix_ascii_case_insensitive(&self, prefix: &str) -> Option<&str> {
        if self.len() >= prefix.len()
        && self[..prefix.len()].eq_ignore_ascii_case(prefix)
        {
            Some(&self[prefix.len()..])
        } else {
            None
        }
    }
}


pub fn parse_http_request_stream(
    stream: &mut TcpStream,
) -> Result<ParsedHttpRequestStream, HttpRequestParsingError> {
    match stream.set_read_timeout(Some(Duration::from_secs(STREAM_READ_TIMEOUT_SECS))) {
        Err(_) => return Err(HttpRequestParsingError::SomeSortOfStreamError),
        _ => {}
    };

    let mut request_line_and_headers: Vec<u8> = Vec::new();
    let mut request_line_and_headers_buf: [u8; 512] = [0; 512];
    loop {
        let bytes_read = match stream.read(&mut request_line_and_headers_buf) {
            Ok(n) => n,
            Err(_) => return Err(HttpRequestParsingError::SomeSortOfStreamError),

        };
        if bytes_read == 0 {
            return Err(HttpRequestParsingError::BadRequest);
        }

        request_line_and_headers.extend_from_slice(&request_line_and_headers_buf[..bytes_read]);

        if request_line_and_headers.len() > MAX_HEADER_BYTES {
            return Err(HttpRequestParsingError::ContentTooLarge);
        }

        if request_line_and_headers.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    let request_line_and_headers_string = String::from_utf8_lossy(&request_line_and_headers);

    let content_length = request_line_and_headers_string
    .lines()
    .find_map(|line| {
        line.strip_prefix_ascii_case_insensitive("Content-Length:")
        .and_then(|v| v.trim().parse::<usize>().ok())
    })
    .unwrap_or(0);

    if content_length > MAX_BODY_BYTES {
        return Err(HttpRequestParsingError::ContentTooLarge);
    }



    let mut body_bytes: Vec<u8> = Vec::with_capacity(content_length.clone());
    let mut body_remaining = content_length.clone();
    let mut body_buf: [u8; 8192] = [0; 8192];

    while body_remaining > 0 {
        let to_read = body_remaining.min(body_buf.len());
        let n = stream.read(&mut body_buf[..to_read])
        .map_err(|_| HttpRequestParsingError::SomeSortOfStreamError)?;

    if n == 0 {
        return Err(HttpRequestParsingError::SomeSortOfStreamError);
    }
    body_bytes.extend_from_slice(&body_buf[..n]);
    body_remaining -= n;

    if body_bytes.len() >= MAX_BODY_BYTES {
        return Err(HttpRequestParsingError::ContentTooLarge);
    }

    }

    Ok(ParsedHttpRequestStream { request_line_and_headers: request_line_and_headers_string.to_string(), body_bytes })
}


pub fn parse_http_request( stream: &mut TcpStream,) -> Result<HttpRequestData, HttpRequestParsingError> {
    let ParsedHttpRequestStream { request_line_and_headers, body_bytes } = parse_http_request_stream(stream)?;
  let mut headers = HashMap::new();

  let mut at_request_line = true;
  let mut request_line = String::from("");
  for line in request_line_and_headers.clone().lines() {
    if at_request_line {
        request_line = line.to_string();
        at_request_line = false;
        continue;
    }
    if let Some((key, value)) = line.split_once(':') {
        headers.insert(key.trim().to_lowercase(), value.trim().to_string());
    }
  }

  let split_request_line: Vec<&str> =  request_line.split_whitespace().collect();
  if split_request_line.len() < 2 {
    return Err(HttpRequestParsingError::BadRequest);
  }
    let method = split_request_line[0];
    let path_with_query = split_request_line[1];

      let cookies = parse_cookies(headers.clone());
     let (path, query) = parse_path_and_query(path_with_query);

     Ok(HttpRequestData { method: method.to_string(), path, query, path_with_query: path_with_query.to_string(), body_bytes, cookies })
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

pub fn parse_username_and_password_from_body(body: String) -> Result<(String, String), String> {
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
