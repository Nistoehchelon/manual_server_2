use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{Read, Write},
    net::{Shutdown, TcpStream}, time::Instant,
};

use crate::http_response::{self, ok_html};

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
pub struct ParsedHeaderStream {
    pub headers: Vec<u8>,
    pub path_and_query: Vec<u8>,
    pub method: Vec<u8>,
}

fn parse_stream(stream: &mut TcpStream) -> Result<ParsedHeaderStream, HttpRequestParsingError> {
//    // stream.set_read_timeout(Some(std::time::Duration::from_secs(7))).unwrap();
//     let mut read_stream: Vec<u8> = vec![];
//     let mut curr_iter = 0;
//     let mut buf = [0u8; 1];
//     let start = Instant::now();
//     let mut encountered_zero_times = 0;
//     dbg!("...");
//     loop {
//         let dur_ms = start.elapsed().as_secs();
//         if dur_ms > 10 {
//             dbg!("timeout", curr_iter, encountered_zero_times );
//         }
//         let read = stream.read(&mut buf).unwrap();
//         if read == 0 {
//             dbg!("Closed");
//             return Err(HttpRequestParsingError::BadRequest);
//         }
//         read_stream.extend_from_slice(&buf[..read]);
//         let read_stream_string = String::from_utf8_lossy(&read_stream);
//         if read_stream_string.contains("\r\n\r\n") {
//             let dur_ms = start.elapsed().as_millis();
//             println!("took {dur_ms} ms and  {curr_iter} iterations and encountered zero from stream.read {encountered_zero_times} times");
//             println!("request: {read_stream_string}");
//             stream.write_all(&ok_html("Ok".to_string())).unwrap();
//             return Err(HttpRequestParsingError::BadRequest);
//         }
//         curr_iter += 1;

//     }
    let mut byte_buf = [0u8; 1];

    let mut body_bytes: Vec<u8> = Vec::new();
    let mut method_bytes: Vec<u8> = Vec::new();
    let mut method_string = String::new();
    let mut path_and_query_bytes: Vec<u8> = Vec::new();
    let mut http_version: Vec<u8> = Vec::new();
    let mut accumulated_rnrn: Vec<u8> = Vec::new();
    let mut header_bytes: Vec<u8> = Vec::new();

    let mut read_header_bytes_len = 0;
    let mut read_method_bytes_len = 0;
    let mut read_total_bytes = 0;



    let mut time_to_parse_method = true;
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
        if time_to_parse_method {
            read_method_bytes_len += 1;
        } else {
            read_header_bytes_len += 1;
        }

        if time_to_parse_method && method_bytes.len() < 4 {
            method_bytes.push(byte_buf[0].clone());
        } else if time_to_parse_method && method_bytes.len() == 4 {
            method_string = match String::from_utf8(method_bytes.clone()) {
                Ok(s) => s,
                Err(e) => {
                    dbg!(format!("Invalid bytes? {e:#?}"));
                    return Err(HttpRequestParsingError::BadRequest);
                }
            };
            if method_string != "GET " && method_string != "POST" {
                dbg!(format!(
                    "Received unknown/unimplemented method {method_string}, aborting"
                ));
                return Err(HttpRequestParsingError::BadRequest);
            }
            method_string = method_string.trim().to_string();
            time_to_parse_method = false;
            time_to_parse_path_and_query = true;
        } else if time_to_parse_method && method_bytes.len() > 4 {
            dbg!("I don't think this can be allowed to happen unless logic errors...");
            return Err(HttpRequestParsingError::BadRequest);
        }

        if time_to_parse_path_and_query {
            if !begun_parsing_path_and_query {
                let byte = byte_buf[0].clone();
                let byte_string = match String::from_utf8(vec![byte]) {
                    Ok(s) => s,
                    Err(e) => {
                        dbg!(format!("Could not convert byte {e:#?}"));
                        return Err(HttpRequestParsingError::BadRequest);
                    }
                };
                if method_string == "POST" && byte_string == " " {
                    time_to_parse_path_and_query = false;
                    time_to_parse_http_version = true;
                } else if method_string == "GET" && byte_string == "/" {
                    path_and_query_bytes.push(byte);
                    time_to_parse_path_and_query = false;
                    time_to_parse_http_version = true;
                } else {
                    dbg!(format!("Malformed request line"));
                    return Err(HttpRequestParsingError::BadRequest);
                }
                begun_parsing_path_and_query = true;
            } else {
                let byte = byte_buf[0].clone();
                let byte_string = match String::from_utf8(vec![byte]) {
                    Ok(s) => s,
                    Err(e) => {
                        dbg!(format!("Could not convert byte {e:#?}"));
                        return Err(HttpRequestParsingError::BadRequest);
                    }
                };
                if byte_string == " " {
                    time_to_parse_http_version = true;
                    time_to_parse_path_and_query = false;
                }
                path_and_query_bytes.push(byte);
            }
        }

        if time_to_parse_http_version {
            let byte = byte_buf[0].clone();
            http_version.push(byte);
            if !http_version.is_ascii() {
                dbg!(format!("non ascii http ver {http_version:#?}"));
                return Err(HttpRequestParsingError::BadRequest);
            }
            let mut http_version_string = String::from_utf8(http_version.clone()).unwrap();
            if http_version_string.contains("\r\n") {
                time_to_parse_http_version = false;
                time_to_parse_headers = true;
                http_version_string = http_version_string.replace("\r\n", "");
                if http_version_string != "HTTP/1.1" {
                    dbg!(format!("Expected HTTP/1.1, got {http_version_string}"));
                    return Err(HttpRequestParsingError::BadRequest);
                }
                continue;
            }
        }

        if time_to_parse_headers {
            let byte = byte_buf[0].clone();
            accumulated_rnrn.push(byte);

            let accumulated_rnrn_string = String::from_utf8_lossy(&accumulated_rnrn);
            if accumulated_rnrn_string.contains("\r\n\r\n") {
                time_to_parse_headers = false;
                finished_parsing_stream = true;
                continue;
            }
            header_bytes.push(byte);
        }

        if read_header_bytes_len > MAX_HEADER_BYTES {
            dbg!(format!(
                "Header bytes length ({read_header_bytes_len}) exceeded MAX_HEADER_BYTES ({MAX_HEADER_BYTES})"
            ));
            return Err(HttpRequestParsingError::ContentTooLarge);
        }
        if finished_parsing_stream {
            break;
        }
    }

    let method_string = match String::from_utf8(method_bytes.clone()) {
        Ok(s) => s,
        Err(e) => {
            dbg!(format!("{e:#?}"));
            return Err(HttpRequestParsingError::BadRequest);
        },
    };
    let path_and_query_string = match String::from_utf8(path_and_query_bytes.clone()) {
        Ok(s) => s,
        Err(e) => {
            dbg!(format!("{e:#?}"));
            return Err(HttpRequestParsingError::BadRequest);
        },
    };

    return Ok(ParsedHeaderStream {
        headers: header_bytes,
        path_and_query: path_and_query_bytes,
        method: method_bytes,
    });
}

pub fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<HttpRequestData, HttpRequestParsingError> {

    let ParsedHeaderStream {
        headers,
        path_and_query,
        method,
    } = parse_stream(stream)?;


    let mut headers_map: HashMap<String, String> = HashMap::new();
    for line in (String::from_utf8_lossy(&headers)).lines() {
        if let Some((key, value)) = line.split_once(':') {
            headers_map.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    let method = String::from_utf8_lossy(&method).to_string();

    let (path, query) = parse_path_and_query(String::from_utf8_lossy(&path_and_query).to_string());

    // --- Parse cookies ---
    let cookies = parse_cookies(headers_map.clone());

    let body: Vec<u8> = vec![];
    Ok(HttpRequestData {
        method: method,
        path,
        path_and_query: String::from_utf8_lossy(&path_and_query).to_string(),
        query,
        body_bytes: body,
        cookies,
        headers: headers_map,
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
