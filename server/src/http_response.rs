pub fn ok_html(content: String) -> String {
    ok(content, "text/html".to_string())
}

pub fn ok(content: String, content_type: String) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: {}; charset=utf-8\r\n\r\n\
            {}",
        content_type, content
    )
}
