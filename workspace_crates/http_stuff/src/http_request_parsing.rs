use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

pub const MAX_HEADER_BYTES: usize = 8 * 1024;      // 8 KB
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB

pub type Query = HashMap<String, String>;

/// Holds all the parsed information from an HTTP request.

#[derive(Clone, Debug)]
pub struct RequestData {
    pub method: String,
    pub path: String,
    pub query: Query,
    pub path_with_query: String,
    pub body: String,
    pub cookies: HashMap<String, String>,
}

pub fn parse_request_data(stream: &mut TcpStream) -> Option<RequestData> {
    let mut reader = BufReader::new(stream);

    // Read the request line
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return None;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0];
    let path_with_query = parts[1];

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // Read body for POST requests
    let mut body = String::new();
    if method == "POST" {
        // Check content-length header
        if let Some(content_length_str) = headers.get("content-length")
            && let Ok(content_length) = content_length_str.parse::<usize>()
        {
            let mut body_bytes = vec![0; content_length];
            if reader.read_exact(&mut body_bytes).is_ok() {
                body = String::from_utf8_lossy(&body_bytes).to_string();
            }
        }
    }

    // Parse cookies
    let cookies = parse_cookies(&headers);

    // Parse path and query
    let (path, query) = parse_path_and_query(path_with_query);

    // url decode
    // I noticed that if pluses are not replaced, if I enter whitespace into input fields, I basically create a user with username like "++++" and is_empty() does not work.
    let body = body.replace('+', " ");
    let body_bytes = urlencoding::decode_binary(body.as_bytes());
    let body = String::from_utf8_lossy(&body_bytes).into_owned();

    let query = query
        .into_iter()
        .map(|(key, value)| {
            let value = value.replace('+', " ");
            let decoded_bytes = urlencoding::decode_binary(value.as_bytes());
            let decoded = String::from_utf8_lossy(&decoded_bytes).into_owned();
            (key, decoded)
        })
        .collect();

    Some(RequestData {
        method: method.to_string(),
        body,
        cookies,
        path_with_query: path_with_query.to_string(),
        query,
        path,
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

fn parse_cookies(headers: &HashMap<String, String>) -> HashMap<String, String> {
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
