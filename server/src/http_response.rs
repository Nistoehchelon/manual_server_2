pub fn ok(content: String) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
            {}",
        content
    )
}
