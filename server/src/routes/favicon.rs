use read_on_dev_embed_on_release::read_on_dev_embed_on_release;

pub fn get() -> Vec<u8> {
    let favicon_bytes = read_on_dev_embed_on_release("favicon.ico");
    let favicon_bytes_len = favicon_bytes.len();
    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: image/x-icon\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\r\n\
        ",
        favicon_bytes_len
    ); 

    let mut response = headers.into_bytes();
    response.extend(favicon_bytes);
    response
}
