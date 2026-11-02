
use http_stuff::{
    http_request_parsing::{HttpMethod, HttpRequestData},
    http_response::{Charset, ContentType, HttpResponseData, ok, ok_html},
};

pub struct StaticResource {
    inner: String,
}
pub struct StaticResourceParseError;
impl StaticResource {
    pub fn as_str(&self) -> &str { &self.inner }
    pub fn parse(string: String) -> Result<Self, StaticResourceParseError> {
        // I guess I will add to this every time I add a static resource?... Or should I just make it -pull it from the static folder dynammicaly, so I could put stuff in on the fly without recompiling? Or, I could even make this .env based and pass a vec-like string in .env??? I dunno...
        let existing_resources = ["app.js", "cloudflare_turnstile_button_enable_disable.js", "stylesheet.css", "favicon.ico"];
        if !existing_resources.contains(&string.as_str()) {
            Err(StaticResourceParseError)
        } else {
            Ok(Self { inner: string })
        }

    }
}
pub fn parse_static_path(path: &str) -> Option<StaticResource> {
    path.strip_prefix("/static/")
        .map(|rest| {
            let r = StaticResource::parse(rest.to_owned());
            r.ok()
        })?
}

pub fn generate_static_response(
    request_data: HttpRequestData,
    static_resources_path: String,
    resource: StaticResource,
) -> HttpResponseData {
    let HttpRequestData { method, .. } = request_data;
    let res_as_str = resource.as_str();
    let full_path = format!("{static_resources_path}/{res_as_str}");
    match (method, res_as_str) {
        (HttpMethod::Get, "favicon.ico") => {
            let Ok(favicon_bytes) = read_bytes(full_path) else {
                return generic_error();
            };

            ok(favicon_bytes, ContentType::ImageXIcon, None)
        }
        (HttpMethod::Get, "stylesheet.css") => {
            let Ok(stylesheet) = read_bytes(full_path) else {
                return generic_error();
            };
            ok(stylesheet, ContentType::TextCss, Some(Charset::Utf8))
        }
        (HttpMethod::Get, "app.js") => {
            let Ok(appjs) = read_bytes(full_path) else {
                return generic_error();
            };
            ok(appjs, ContentType::TextJavascript, Some(Charset::Utf8))
        }
        (HttpMethod::Get, "cloudflare_turnstile_button_enable_disable.js") => {
            let Ok(script) = read_bytes(full_path) else {
                return generic_error();
            };
            ok(script, ContentType::TextJavascript, Some(Charset::Utf8))
        }
        _ => generic_error(),
    }
}

fn generic_error() -> HttpResponseData {
    ok_html("Static resource not found, yay!".to_string())
}

fn read_bytes(path: String) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}
