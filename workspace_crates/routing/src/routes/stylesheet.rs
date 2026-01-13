use http_stuff::http_response::*;
use read_on_dev_embed_on_release::read_to_string_on_dev_embed_on_release;

pub fn get() -> Result<ResponseData, error_structs::InternalAppError> {
    let stylesheet_bytes = get_stylesheet_css_content().into_bytes();

    Ok(ResponseData {
        status_code: StatusCode::Ok200,
        content_type: ContentType::TextCss,
        charset: Some(Charset::Utf8),
        body: Some(stylesheet_bytes),
        set_cookie_headers: None,
        location: None,
    })
}

fn get_stylesheet_css_content() -> String {
    let stylesheet = read_to_string_on_dev_embed_on_release("stylesheet.css");

    stylesheet
}
