pub fn to(page: String) -> String {
    format!(
        "HTTP/1.1 302 Found\r\n\
         Location: {page}\r\n\
         Content-Type: text/html\r\n\r\n"
    )
}
