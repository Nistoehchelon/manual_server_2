use http_stuff::http_response::{self, ResponseData};
use read_on_dev_embed_on_release::read_on_dev_embed_on_release;

pub fn get() -> Result<ResponseData, error_structs::InternalAppError> {
    let favicon_bytes = read_on_dev_embed_on_release("favicon.ico");
    let response = http_stuff::http_response::ok(favicon_bytes, http_response::ContentType::ImageXIcon);
    Ok(response)
}
