use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{BufReader, Read},
    net::TcpStream,
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
pub struct ParsedHeaderStream {
    pub headers: Vec<u8>,
    pub leftover: Vec<u8>,
}


fn parse_headers_stream(
    stream: &mut TcpStream,
) -> Result<ParsedHeaderStream, HttpRequestParsingError> {
    const BYTES_PER_READ: usize = 1024;

    let mut byte_buf = [0u8; BYTES_PER_READ];
    let mut header_bytes = Vec::new();
    let mut read_total_bytes = 0;

    loop {
        let bytes_read = match stream.read(&mut byte_buf) {
            Ok(0) => {
                dbg!("Connection closed? 0 bytes read before headers completed");
                return Err(HttpRequestParsingError::BadRequest);
            },

            Ok(n) => {
                
                n
            },
            Err(e) => {
                dbg!("Stream read error", e);
                return Err(HttpRequestParsingError::SomeSortOfStreamError);
            },
        };

        read_total_bytes += bytes_read;

        if read_total_bytes > MAX_HEADER_BYTES {
            dbg!("Header size exceeded max allowed", read_total_bytes, MAX_HEADER_BYTES);
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

            return Ok(ParsedHeaderStream { headers: header_bytes, leftover: leftover_body });
        }
    }
}




pub fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<HttpRequestData, HttpRequestParsingError> {

    let ParsedHeaderStream {
        headers,
        leftover: leftover_from_headers_parsing,
    } = parse_headers_stream(stream)?;


    let mut headers_hash_map: HashMap<String, String> = HashMap::new();
    for line in (String::from_utf8_lossy(&headers)).lines() {
        if let Some((key, value)) = line.split_once(':') {
            headers_hash_map.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    let binding = ((String::from_utf8_lossy(&headers)).lines().collect::<Vec<&str>>()[0]).to_string();
    let first_line_split = binding.split(" ").collect::<Vec<&str>>();
    if first_line_split.len() != 3 {
        dbg!(format!("I expected something other than {first_line_split:#?} for first header line"));
        return Err(HttpRequestParsingError::BadRequest);
    }
    let method = first_line_split[0].to_string();
    let path_and_query = first_line_split[1].to_string();
    let http_version = first_line_split[2].to_string();

    dbg!(http_version);

    let (path, query) = parse_path_and_query(path_and_query.clone());

    // --- Parse cookies ---
    let cookies = parse_cookies(headers_hash_map.clone());

    let mut body: Vec<u8> = vec![];

    let mut content_length = match headers_hash_map.get("content-length").unwrap_or(&"0".to_string()).parse::<usize>() {
        Err(e) => {
            dbg!("Parse error", e);
            return Err(HttpRequestParsingError::BadRequest);
        },
        Ok(v) => v,
    };
    if content_length > MAX_BODY_BYTES {
        dbg!(content_length, "exceeds", MAX_BODY_BYTES);
        return Err(HttpRequestParsingError::ContentTooLarge);
    } else if method == "POST" {
        if leftover_from_headers_parsing.len() > 0 {
        if leftover_from_headers_parsing.len() > content_length {
            dbg!("Leftover from header reading is greater than promised content length, rejecting", leftover_from_headers_parsing.len(), content_length);
            return Err(HttpRequestParsingError::BadRequest);
        }
        content_length -= leftover_from_headers_parsing.len();
        body.extend(leftover_from_headers_parsing);
        } 
        if content_length > 0 {
        let mut reader = BufReader::new(stream);
        let mut body_bytes_exact = vec![0; content_length];
        match reader.read_exact(&mut body_bytes_exact) {
            Err(e) => {
                dbg!("read_exact error body", e);
                return Err(HttpRequestParsingError::BadRequest);
            },
            Ok(_) => {
            },
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
        headers: headers_hash_map,
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
