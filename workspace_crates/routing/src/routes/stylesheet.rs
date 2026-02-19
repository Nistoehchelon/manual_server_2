use http_stuff::http_response::*;
use read_on_dev_embed_on_release::read_to_string_on_dev_embed_on_release;

pub fn get() -> Result<HttpResponseData, error_structs::InternalAppError> {
    let stylesheet_bytes = get_stylesheet_css_content().into_bytes();

    Ok(HttpResponseData {
        body: Some(stylesheet_bytes),

        headers: Headers {
            status_code: StatusCode::Code200Ok,
            content_type: ContentType::TextCss,
            charset: Some(Charset::Utf8),
            set_cookie: None,
            location: None,
            connection: KeepAliveOrClose::KeepAlive,
            other_headers: None,
        },
    })
}

fn get_stylesheet_css_content() -> String {
    read_to_string_on_dev_embed_on_release("stylesheet.css")
}
