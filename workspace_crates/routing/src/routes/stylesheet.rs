use http_stuff::http_response::*;
use read_on_dev_embed_on_release::read_to_string_on_dev_embed_on_release;

pub fn get() -> Result<HttpResponseData, error_structs::InternalAppError> {
    let stylesheet_bytes = get_stylesheet_css_content().into_bytes();

    Ok(HttpResponseData {
        body: Some(stylesheet_bytes),
        status_code: StatusCode::Code200Ok,
        set_cookie: None,
        headers: {
            let mut headers = http_stuff::http_response::Headers::new();

            headers.insert("content-type", ContentType::TextCss.to_string());
            headers.insert("charset", Charset::Utf8.to_string());
            headers.insert("connection", KeepAliveOrClose::KeepAlive.to_string());
            headers
        },
    })
}

fn get_stylesheet_css_content() -> String {
    read_to_string_on_dev_embed_on_release("stylesheet.css")
}
