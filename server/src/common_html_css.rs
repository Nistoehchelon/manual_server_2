use crate::{
    read_resource_file_on_dev_or_embed_on_release::read_to_string_on_dev_embed_on_release, storage,
};

pub fn get_header_html(title: String) -> String {
    let html_header = read_to_string_on_dev_embed_on_release(storage::HEADER_FILE_NAME);
    let html_header = html_header.replace("[// title //]", title.as_str());

    html_header
}

pub fn get_footer_html() -> String {
    let html_footer = read_to_string_on_dev_embed_on_release(storage::FOOTER_FILE_NAME);

    html_footer
}

pub fn get_stylesheet_css() -> String {
    let stylesheet = read_to_string_on_dev_embed_on_release(storage::STYLESHEET_FILE_NAME);

    stylesheet
}

pub fn surround_content_with_header_and_footer(content: String, title: String) -> String {
    format!(
        r##"
    {}
    {}
    {}"##,
        get_header_html(title),
        content,
        get_footer_html()
    )
}
