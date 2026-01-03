use read_on_dev_embed_on_release::read_on_dev_embed_on_release;

pub fn get() -> Vec<u8> {
    let favicon_bytes = read_on_dev_embed_on_release("favicon.ico");
    let response = http_stuff::http_response::ok(favicon_bytes, String::from("image/x-icon"));
    response
}
