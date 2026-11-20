use http_stuff::http_response::{Charset, HttpResponseData, ok};

use routing_structs::RouterData;
use user::username_color::get_allowed_username_colors;

// /dynamic_stylesheet/contact?offset=0
pub async fn get(router_data: RouterData) -> anyhow::Result<HttpResponseData> {
    let path = router_data.http_request_data.path.clone();
    let Some(page) = path.strip_prefix("/dynamic_stylesheet/") else {
        tracing::debug!("Could not strip prefix");
        return crate::not_found::not_found(router_data);
    };
    let page = page.to_string();
    let Some(stylesheet_number) = router_data
        .http_request_data
        .query
        .get("stylesheet_number")
        .and_then(|s| s.parse::<u64>().ok())
    else {
        tracing::debug!("Could not  extract stylesheet_number");
        return crate::not_found::not_found(router_data);
    };
    let Ok(content) = dynamic_stylesheets::get_content(
        router_data.db_pool.clone(),
        page.clone(),
        stylesheet_number,
    )
    .await
    else {
        tracing::debug!(
            "Could not get content for page={page} stylesheet_number={stylesheet_number}"
        );
        return crate::not_found::not_found(router_data);
    };
    let resp = ok(
        content.into_bytes(),
        http_stuff::http_response::ContentType::TextCss,
        Some(Charset::Utf8),
    );
    Ok(resp)
}

pub async fn allowed_username_colors(router_data: RouterData) -> anyhow::Result<HttpResponseData> {
    let conn = &mut router_data.db_pool.acquire().await?;
    let colors = get_allowed_username_colors(conn).await?;
    let mut css = String::new();
    for c in colors {
        let color_class = c.strip_prefix("#").unwrap_or(&c);
        let append = format!(
            r###"
.username_color_{color_class} {{
    color: {c};
}}
"###
        );
        css.push_str(&append);
    }
    let resp = ok(
        css.into_bytes(),
        http_stuff::http_response::ContentType::TextCss,
        Some(Charset::Utf8),
    );
    Ok(resp)
}
