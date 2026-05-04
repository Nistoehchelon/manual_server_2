use http_stuff::http_response::{Charset, HttpResponseData, ok};

use crate::RouterData;

// /dynamic_stylesheet/contact?offset=0
pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let path = router_data.http_request_data.path.clone();
    let Some(page) = path.strip_prefix("/dynamic_stylesheet/") else {
        tracing::debug!("Could not strip prefix");
        return crate::routes::not_found::not_found(router_data);
    };
    let page = page.to_string();
    let Some(stylesheet_number) = router_data
        .http_request_data
        .query
        .get("stylesheet_number")
        .and_then(|s| s.parse::<u64>().ok())
    else {
        tracing::debug!("Could not  extract stylesheet_number");
        return crate::routes::not_found::not_found(router_data);
    };
    let Ok(content) =
        dynamic_stylesheets::get_content(router_data.db_pool.clone(), page.clone(), stylesheet_number)
            .await
    else {
        tracing::debug!("Could not get content for page={page} stylesheet_number={stylesheet_number}");
        return crate::routes::not_found::not_found(router_data);
    };
    let resp = ok(
        content.into_bytes(),
        http_stuff::http_response::ContentType::TextCss,
        Some(Charset::Utf8),
    );
    Ok(resp)
}
