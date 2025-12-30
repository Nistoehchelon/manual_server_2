use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let request: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap_or_default())
        .take_while(|line| !line.is_empty())
        .collect();

    if request.is_empty() {
        return;
    }

    let first_line = &request[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path_with_query = parts[1];

    // Parse path and query parameters
    let (path, query_params) = parse_path_and_query(path_with_query);

    // Parse headers
    let headers = parse_headers(&request);

    // Parse cookies from headers
    let cookies = parse_cookies(&headers);

    // Handle routes
    let response = match (method, path.as_str()) {
        ("GET", "/cookie") => {
            // Display cookie value
            let cookie_value = cookies.get("hm").unwrap_or(&"not set".to_string()).clone();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                <h1>Cookie Value</h1>\
                <p>hm = {}</p>",
                cookie_value
            )
        }
        ("GET", "/set_cookie") => {
            // Extract key and value from query parameters
            let key = query_params.get("key").cloned().unwrap_or_default();
            let value = query_params.get("value").cloned().unwrap_or_default();
            
            // Set cookie in response
            format!(
                "HTTP/1.1 200 OK\r\n\
                Set-Cookie: {}={}; Path=/\r\n\
                Content-Type: text/html\r\n\r\n\
                <h1>Cookie Set</h1>\
                <p>Set cookie: {} = {}</p>\
                <p><a href='/cookie'>View cookie</a></p>",
                key, value, key, value
            )
        }
        _ => {
            // Default response
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                <h1>Simple HTTP Server</h1>\
                <p>Endpoints:</p>\
                <ul>\
                <li><a href='/cookie'>/cookie</a> - Show cookie value</li>\
                <li><a href='/set_cookie?key=hm&value=haha'>/set_cookie?key=hm&value=haha</a> - Set cookie</li>\
                </ul>"
            )
        }
    };

    stream.write_all(response.as_bytes()).ok();
}

fn parse_path_and_query(path_with_query: &str) -> (String, HashMap<String, String>) {
    let parts: Vec<&str> = path_with_query.split('?').collect();
    let path = parts[0].to_string();
    let mut query_params = HashMap::new();

    if parts.len() > 1 {
        for pair in parts[1].split('&') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    (path, query_params)
}

fn parse_headers(request: &[String]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in request.iter().skip(1) {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    headers
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