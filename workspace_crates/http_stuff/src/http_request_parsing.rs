use config_reader::Config;
use encoding::decode_request_string;
use indexmap::IndexMap;
use std::{
    collections::HashMap, fmt::{self, Display}, time::Duration
};
use tokio::io::BufReader;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    time::timeout,
};
use url::Query;

#[derive(Debug, Clone)]
pub struct HttpParserConfig {
    pub read_timeout: Duration,
    pub max_headers_len: u64,
    pub max_header_lines_count: u64,
    pub max_header_name_len: u64,
    pub max_header_value_len: u64,
    pub max_body_len: u64,
    pub max_request_target_length: u64,
    pub max_cookies_per_request_count: u64,
}
impl From<Config> for HttpParserConfig {
    fn from(server_config: Config) -> Self {
        Self {
            read_timeout: server_config.common_stream_timeout,
            max_headers_len: server_config.max_total_headers_size_bytes,
            max_header_lines_count: server_config.max_headers_count,
            max_header_name_len: server_config.max_header_name_len,
            max_header_value_len: server_config.max_header_value_len,
            max_body_len: server_config.max_total_body_size_bytes,
            max_request_target_length: server_config.max_request_path_length,
            max_cookies_per_request_count: server_config.max_cookies_count,
        }
    }
}

pub async fn parse_http_request<S: AsyncRead + Unpin>(
    reader: &mut BufReader<S>,
    parser_config: HttpParserConfig,
) -> Result<HttpRequestData, HttpRequestParsingError> {
    let ParsedHeaderStream { headers } =
        parse_headers_from_stream(reader, parser_config.clone()).await?;
    let headers_string = String::from_utf8_lossy(&headers).to_string();
    let headers_count = count_occurences_of_rn(&headers_string);

    if headers_count > parser_config.max_header_lines_count {
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
        if key.len() as u64 > parser_config.max_header_name_len {
            return Err(HttpRequestParsingError::HeaderNameTooLong);
        }
        if value.len() as u64 > parser_config.max_header_value_len {
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
        dbg!(http_version.clone());
        return Err(HttpRequestParsingError::HttpVersionNotSupported);
    }

    if path_and_query.len() as u64 > parser_config.max_request_target_length {
        return Err(HttpRequestParsingError::UriTooLong);
    }
    if !request_target_characters_are_valid(&path_and_query) {
        return Err(HttpRequestParsingError::BadRequest);
    }

    let (path, query) = parse_path_and_query(path_and_query.clone());

    // --- Parse cookies ---
    let cookies = parse_cookies(headers.clone(), parser_config.max_cookies_per_request_count)?;

    let mut body: Vec<u8> = vec![];

    let content_length = match headers.get("content-length").unwrap_or("0").parse::<u64>() {
        Err(e) => {
            dbg!("Parse error", e);
            return Err(HttpRequestParsingError::BadRequest);
        }
        Ok(v) => v,
    };
    if content_length > parser_config.max_body_len {
        dbg!(content_length, "exceeds", parser_config.max_body_len);
        return Err(HttpRequestParsingError::ContentTooLarge);
    } else if method == "POST" && content_length > 0 {
        let mut body_bytes_exact = vec![0; content_length.try_into().unwrap()];
        match timeout(
            parser_config.read_timeout,
            reader.read_exact(&mut body_bytes_exact),
        )
        .await
        {
            Ok(Err(e)) => {
                dbg!("read_exact error body", e);
                return Err(HttpRequestParsingError::BadRequest);
            }
            Ok(Ok(_)) => {}
            Err(e) => {
                dbg!("Timeout err read_exact body ", e);
            }
        }
        body.extend(body_bytes_exact);
    }

    Ok(HttpRequestData {
        method,
        path,
        path_and_query,
        query,
        body_bytes: body,
        cookies,
        headers,
    })
}

async fn read_line_bounded<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    max_line_len: u64,
    timeout_dur: Duration,
    header_bytes_so_far: Vec<u8>,
) -> Result<Vec<u8>, HttpRequestParsingError> {
    let mut line = Vec::new();

    loop {
        let byte = timeout(timeout_dur, reader.read_u8())
            .await
            .map_err(|elapsed| {
                if header_bytes_so_far.is_empty() {
                    dbg!("client timeout before client sent any bytes");
                    return HttpRequestParsingError::ClientTimedOutBeforeItSentAnyBytes;
                }
                dbg!(
                    "header byte reading timed out",
                    elapsed,
                    header_bytes_so_far.to_ascii_lowercase()
                );
                HttpRequestParsingError::HeadersStreamReadTimeout
            })?
            .map_err(|e| {
                if header_bytes_so_far.is_empty() {
                    dbg!("nothing from client...", e);
                    return HttpRequestParsingError::ClientSentNothing;
                }
                dbg!(
                    "header byte reading failed",
                    e,
                    header_bytes_so_far.to_ascii_lowercase()
                );
                HttpRequestParsingError::SomeSortOfStreamError
            })?;

        line.push(byte);

        if line.len() as u64 > max_line_len {
            dbg!(
                "Header line parsing: line exceeded max_line_len, rejecting request",
                line.len(),
                max_line_len
            );
            return Err(HttpRequestParsingError::HeaderValueTooLong);
        }

        if line.ends_with(b"\n") {
            break;
        }
    }

    Ok(line)
}

async fn parse_headers_from_stream<S: AsyncRead + Unpin>(
    reader: &mut BufReader<S>,
    parser_config: HttpParserConfig,
) -> Result<ParsedHeaderStream, HttpRequestParsingError> {
    let mut header_bytes = Vec::new();
    let mut read_total_bytes = 0;

    loop {
        let line = read_line_bounded(
            reader,
            parser_config.max_header_name_len + parser_config.max_header_value_len,
            parser_config.read_timeout,
            header_bytes.clone(),
        )
        .await?;
        read_total_bytes += line.len();

        if read_total_bytes as u64 > parser_config.max_headers_len {
            dbg!(
                "Header size exceeded max allowed",
                read_total_bytes,
                parser_config.max_headers_len
            );
            return Err(HttpRequestParsingError::ContentTooLarge);
        }

        // Append the bytes we just read
        header_bytes.extend(&line);

        // Check for header-body boundary
        if header_bytes.ends_with(b"\r\n\r\n") {
            // not sure if there is a point to this check... Vec u8 will by definition only contain ascii  bytes, right?
            if !header_bytes.is_ascii() {
                dbg!("I do not accept headers with non-ascii bytes");
                return Err(HttpRequestParsingError::BadRequest);
            }
            break;
        }
    }
    Ok(ParsedHeaderStream {
        headers: header_bytes,
    })
}

/// Holds all the parsed information from an HTTP request.
#[derive(Clone, Debug)]
pub struct HttpRequestData {
   // pub ip: IpAddr,
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
    ClientTimedOutBeforeItSentAnyBytes,
    ClientSentNothing,
    HeadersStreamReadTimeout,
    TooManyCookies,
    HttpVersionNotSupported,
    UriTooLong,
    StreamTimeoutError,
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
            Self::ClientTimedOutBeforeItSentAnyBytes => "Client Timed Out Before It Sent Any Bytes",
            Self::ClientSentNothing => "Client Sent Nothing",
            Self::HeadersStreamReadTimeout => "Headers Stream Read Timeout",
            Self::BadRequest => "Bad Request",
            Self::ContentTooLarge => "Content Too Large",
            Self::SomeSortOfStreamError => "Some Sort Of Stream Error",
            Self::StreamTimeoutError => "Stream Timeout Error",
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
                return Err("Could not split by = in post".to_string());
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
    Ok(params)
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

fn parse_cookies(
    headers: Headers,
    max_cookies_per_request_count: u64,
) -> Result<HashMap<String, String>, HttpRequestParsingError> {
    if let Some(cookie_header) = headers.get("cookie") {
        let cookies_count = cookie_header.matches(";").count();
        if cookies_count as u64 > max_cookies_per_request_count {
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

pub fn count_occurences_of_rn(s: &str) -> u64 {
    s.match_indices("\r\n").count() as u64
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
