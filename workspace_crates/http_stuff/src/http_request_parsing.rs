use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::Read,
    net::TcpStream,
    time::Duration,
};

pub const MAX_HEADER_BYTES: usize = 8 * 1024; // 8 KB
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
pub const STREAM_READ_TIMEOUT_SECS: u64 = 8;

pub type Query = HashMap<String, String>;

/// Holds all the parsed information from an HTTP request.
#[derive(Clone, Debug)]
pub struct HttpRequestData {
    pub headers: HashMap<String, String>,
    pub method: String,
    pub path: String,
    pub query: Query,
    pub path_and_query: String,
    pub body_bytes: Vec<u8>,
    pub cookies: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum HttpRequestParsingError {
    SomeSortOfStreamError,
    ContentTooLarge,
    BadRequest,
    UnexpectedEof,
}
impl Display for HttpRequestParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::BadRequest => "Bad Request",
            Self::ContentTooLarge => "Content Too Large",
            Self::SomeSortOfStreamError => "Some Sort Of Stream Error",
            Self::UnexpectedEof => "Unexpected End Of File",
        };
        write!(f, "{}", s)
    }
}
pub struct ParsedStream {
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub path_and_query: String,
    pub method: String,
}

fn parse_stream(stream: &mut TcpStream) -> Result<ParsedStream, HttpRequestParsingError> {
    let mut byte_buf = [0u8; 1];

    let mut body_bytes: Vec<u8> = Vec::new();
    let mut request_line_bytes: Vec<u8> = Vec::new();
    let mut method = String::from("");
    let mut path_and_query = String::from("");
    let mut http_version = String::from("");
    let mut accumulated_rnrn = String::from("");
    let mut headers_string: String = String::from("");
    let mut headers_hash_map: HashMap<String, String> = HashMap::new();

    let mut read_header_bytes_len = 0;
    let mut read_body_bytes_len = 0;
    let mut read_request_line_bytes = 0;
    let mut read_total_bytes = 0;



    let mut time_to_parse_method = false;
    let mut time_to_parse_path_and_query = false;
    let mut time_to_parse_http_version = false;
    let mut begun_parsing_path_and_query = false;
    let mut time_to_parse_headers = false;
    let mut time_to_parse_body = false;
    let mut finished_parsing_stream = false;
    loop {
        let bytes_read = match stream.read(&mut byte_buf) {
            Ok(n) => n,
            Err(e) => {
                dbg!(format!("Stream read error: {e:#?}"));
                return Err(HttpRequestParsingError::SomeSortOfStreamError);
            }
        };
        if bytes_read == 0 {
            dbg!("Hm, read 0 bytes?", read_total_bytes);
            return Err(HttpRequestParsingError::BadRequest);
        }

        if !byte_buf.is_ascii() {
            dbg!("Got non ascii header, aborting");
            return Err(HttpRequestParsingError::BadRequest);
        }
        read_total_bytes += 1;
        if time_to_parse_body {
            read_body_bytes_len += 1;
        } else {
            read_header_bytes_len += 1;
        }

        if read_request_line_bytes < 4 {
            read_request_line_bytes += 1;
            request_line_bytes.push(byte_buf[0].clone());
        } else if time_to_parse_method && read_header_bytes_len == 4 {
            method = match String::from_utf8(request_line_bytes.clone()) {
                Ok(s) => s,
                Err(_) => {
                    dbg!("Invalid bytes?");
                    return Err(HttpRequestParsingError::BadRequest);
                }
            };
            if method != "GET " || method != "POST" {
                dbg!(format!(
                    "Received unknown/unimplemented method {method}, aborting"
                ));
                return Err(HttpRequestParsingError::BadRequest);
            }
            method = method.trim().to_string();
            time_to_parse_method = false;
            time_to_parse_path_and_query = true;
        } else if time_to_parse_method && read_header_bytes_len > 4 {
            dbg!("I don't think this can be allowed to happen unless logic errors...");
            return Err(HttpRequestParsingError::BadRequest);
        }

        if time_to_parse_path_and_query {
            if !begun_parsing_path_and_query {
                let byte = byte_buf[0].clone() as char;
                if method == "POST" && byte == ' ' {
                    time_to_parse_path_and_query = false;
                    time_to_parse_http_version = true;
                } else if method == "GET" && byte == '/' {
                    path_and_query.push(byte);
                    time_to_parse_path_and_query = false;
                    time_to_parse_http_version = true;
                } else {
                    dbg!(format!("Malformed request line"));
                    return Err(HttpRequestParsingError::BadRequest);
                }
                begun_parsing_path_and_query = true;
            } else {
                let byte = byte_buf[0].clone() as char;
                if byte == ' ' {
                    time_to_parse_http_version = true;
                    time_to_parse_path_and_query = false;
                }
                path_and_query.push(byte);
            }
        }

        if time_to_parse_http_version {
            let byte = byte_buf[0].clone() as char;
            http_version.push(byte);
            if http_version.contains("\r\n") {
                time_to_parse_http_version = false;
                time_to_parse_headers = true;
                http_version = http_version.replace("\r\n", "");
                if http_version != "HTTP/1.1" {
                    dbg!(format!("Expected HTTP/1.1, got {http_version}"));
                    return Err(HttpRequestParsingError::BadRequest);
                }
                continue;
            }
        }

        if time_to_parse_headers {
            let byte = byte_buf[0].clone() as char;
            accumulated_rnrn.push(byte);
            if accumulated_rnrn.contains("\r\n\r\n") {
                time_to_parse_headers = false;
                time_to_parse_body = true;
                let headers_string_clone = headers_string.clone();
                for line in headers_string_clone.split("\r\n") {
                    if let Some((key, value)) = line.split_once(':') {
                        headers_hash_map
                            .insert(key.trim().to_lowercase(), value.trim().to_string());
                    } else {
                        dbg!(format!("Incorrect header value {line}"));
                        return Err(HttpRequestParsingError::BadRequest);
                    }
                }
                match headers_hash_map.get("transfer-encoding") {
                    Some(_) => {
                        dbg!("Client sent transfer-encoding which I did not implement");
                        return Err(HttpRequestParsingError::BadRequest);
                    }
                    _ => {}
                }

                continue;
            }
            headers_string.push(byte);
        }

        if time_to_parse_body {
            let content_length = match headers_hash_map.get("content-length") {
                Some(v) => v,
                None => "0",
            };
            let content_length: usize = match content_length.parse() {
                Ok(n) => n,
                Err(_) => {
                    dbg!(format!(
                        "Could not convert content_length's value ({content_length}) into number, rejecting request"
                    ));
                    return Err(HttpRequestParsingError::BadRequest);
                }
            };
            if bytes_read != 0 && body_bytes.len() >= content_length {
                dbg!(
                    "Client is sending more bytes than it promised to send using content-length header, rejecting"
                );
                return Err(HttpRequestParsingError::BadRequest);
            } else if bytes_read == 0 && body_bytes.len() == content_length {
                time_to_parse_body = false;
                finished_parsing_stream = true;
                continue;
            }
            let byte = byte_buf[0];
            body_bytes.push(byte);
        }

        if read_header_bytes_len > MAX_HEADER_BYTES {
            dbg!(format!(
                "Header bytes length ({read_header_bytes_len}) exceeded MAX_HEADER_BYTES ({MAX_HEADER_BYTES})"
            ));
            return Err(HttpRequestParsingError::ContentTooLarge);
        }
        if read_body_bytes_len > MAX_BODY_BYTES {
            dbg!(format!(
                "Body bytes length ({read_body_bytes_len}) exceeded MAX_BODY_BYTES ({MAX_BODY_BYTES})"
            ));
            return Err(HttpRequestParsingError::ContentTooLarge);
        }
        if finished_parsing_stream {
            break;
        }
    }

    return Ok(ParsedStream {
        body: body_bytes.clone(),
        headers: headers_hash_map.clone(),
        path_and_query: path_and_query,
        method: method.clone(),
    });
}

pub fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<HttpRequestData, HttpRequestParsingError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(STREAM_READ_TIMEOUT_SECS)))
        .map_err(|_| {
            dbg!("Could not set timeout :(");
            HttpRequestParsingError::SomeSortOfStreamError
        })?;

    let ParsedStream {
        body,
        headers,
        path_and_query,
        method,
    } = parse_stream(stream)?;

    let (path, query) = parse_path_and_query(path_and_query.clone());

    // --- Parse cookies ---
    let cookies = parse_cookies(headers.clone());

    Ok(HttpRequestData {
        method,
        path,
        path_and_query: path_and_query.to_string(),
        query,
        body_bytes: body,
        cookies,
        headers,
    })
}

fn parse_path_and_query(path_with_query: String) -> (String, HashMap<String, String>) {
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
    let body = String::from_utf8_lossy(&body);
    let body = match urlencoding::decode(&body) {
        Err(_) => {
            return Err(format!("Could not decode body into string"));
        }
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
