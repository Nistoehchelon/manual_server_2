use crate::common_html_css;

pub fn get() -> String {
    let stylesheet = common_html_css::get_stylesheet_css();
    format!(
        "HTTP/1.1 200 OK\
      Content-Type: text/css\r\n\r\n\
      {stylesheet}"
    )
}
