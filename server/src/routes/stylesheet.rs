use read_on_dev_embed_on_release::read_to_string_on_dev_embed_on_release;

pub fn get() -> String {
    let stylesheet = get_stylesheet_css_content();
    format!(
        "HTTP/1.1 200 OK\
      Content-Type: text/css\r\n\r\n\
      {stylesheet}"
    )
}

fn get_stylesheet_css_content() -> String {
    let stylesheet = read_to_string_on_dev_embed_on_release(storage::STYLESHEET_FILE_NAME);

    stylesheet
}
