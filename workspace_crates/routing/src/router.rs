use config_reader::Config;
use html_templating::html_template_structs::BaseTemplateData;
use into_vec::into_vec;
use std::time::Instant;

use error_structs::InternalAppError;
use http_stuff::{
    http_request_parsing::HttpRequestData,
    http_response::{
        self, Charset, ContentType, HttpResponseData, KeepAliveOrClose, SetCookieHeader,
        StatusCode, internal_server_error_500, redirect_to, standalone_response,
    },
};
use session::{
    self, Session, SessionNotification, get_session_by_session_id, set_previous_page_load_time_ms,
};
use user::{self, get_user_by_user_id};

use crate::routes;

#[derive(Clone)]
pub struct RouterData {
    pub server_config: Config,
    pub db_pool: sqlx::PgPool,
    pub session: Session,
    pub http_request_data: HttpRequestData,
    pub base_template_data: BaseTemplateData,
    pub session_notifications: Vec<SessionNotification>,
}

pub async fn route(
    db_pool: sqlx::PgPool,
    server_config: Config,
    session: Session,
    created_new_session: bool,
    request_data: HttpRequestData,
    base_template_data: BaseTemplateData,
) -> Result<HttpResponseData, InternalAppError> {
    let HttpRequestData {
        path_and_query: _path_and_query,
        path,
        method,
        query: _,
        body_bytes: _,
        ..
    } = request_data.clone();

    let mut response_data: HttpResponseData = 'response_block: {
        if session.logged_in {
            let user_associated_with_session_actually_exists =
                get_user_by_user_id(db_pool.clone(), session.user_id.clone())
                    .await
                    .map_err(InternalAppError::new)
                    .is_ok();
            if !user_associated_with_session_actually_exists {
                let mut modified_session = session.clone();
                modified_session.logged_in = false;
                modified_session.user_id = "".to_string();
                session::edit_session(
                    db_pool.clone(),
                    modified_session.id.clone(),
                    modified_session,
                )
                .await
                .map_err(InternalAppError::new)?;
                break 'response_block redirect_to("/".to_string());
            }
        }
        let session_notifications =
            session::read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
                .await
                .map_err(InternalAppError::new)?;
        let router_data = RouterData {
            session_notifications,
            server_config: server_config.clone(),
            db_pool: db_pool.clone(),
            session: session.clone(),
            http_request_data: request_data.clone(),
            base_template_data,
        };
        match (method.as_str(), path.as_str()) {
            ("GET", "/signup") => routes::signup::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("POST", "/signup") => routes::signup::post(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/dashboard") => routes::dashboard::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/logout") => routes::logout::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/login") => routes::login::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("POST", "/login") => routes::login::post(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/forum") => routes::forum::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/create_thread") => routes::create_thread::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("POST", "/create_thread") => routes::create_thread::post(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/forum_thread") => routes::forum_thread::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("POST", "/forum_thread") => routes::forum_thread::post(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("GET", "/forum_thread_message_versions") => {
                routes::forum_thread_message_versions::get(router_data.clone())
                    .await
                    .unwrap_or(internal_server_error_500())
            },
            ("GET", "/edit_forum_thread_message") => {
                routes::edit_forum_thread_message::get(router_data.clone()).await.unwrap_or(internal_server_error_500())
            },
            ("POST", "/edit_forum_thread_message") => {
                routes::edit_forum_thread_message::post(router_data.clone()).await.unwrap_or(internal_server_error_500())
            },
            ("GET", "/create_forum") => {
                routes::create_forum::get(router_data.clone()).await.unwrap_or(internal_server_error_500())
            },
            ("POST", "/create_forum") => {
                routes::create_forum::post(router_data.clone()).await.unwrap_or(internal_server_error_500())
            },
            ("POST", "/dump_request") => routes::dump_request::dump_request(router_data.clone())
                .unwrap_or(internal_server_error_500()),

            ("GET", "/manage_site_pages") => routes::manage_custom_site_pages::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
            ("POST", "/webhook_and_api") => {
                routes::webhook_and_api::post(router_data.clone()).await.unwrap_or(internal_server_error_500())
            },
            (method, path) => {
                let mut path = path.to_string();
                path.remove(0); // remove slash
                if !path.is_empty() && let Ok(_thread) = forum::get_thread(db_pool.clone(), path.clone()).await {
                    routes::view_custom_site_page::get(router_data.clone(), path).await.unwrap_or(internal_server_error_500())
                } else if !path.is_empty() {
                    routes::not_found::not_found(router_data.clone(), method.to_string(), path.clone())
                } else {
                routes::index::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500())
                }
            },
        }
    };

    if created_new_session {
        let ch = SetCookieHeader {
            key: session::get_name_of_cookie_containing_session_id(),
            value: session.id.clone(),
            path: "/".to_string(),
            secure: server_config.using_https,
        };
        dbg!(ch.clone());
        match response_data.set_cookie {
            None => {
                let v = vec![ch];
                response_data.set_cookie = Some(v);
            }
            Some(ref mut sch) => {
                sch.push(ch);
            }
        }
    }

    Ok(response_data)
}

pub async fn response_to_bytes(response_data: HttpResponseData) -> Vec<u8> {
    match response_data.response() {
        Ok(rb) => rb,
        Err(_) => standalone_response(StatusCode::Code500InternalServerError, "Error".to_string()),
    }
}
pub async fn generate_response_data(
    request_data: HttpRequestData,
    db_pool: sqlx::PgPool,
    server_config: Config,
) -> HttpResponseData {
    let nonce = id_generator::generate_random_alphanumeric_id(50).to_string();
    if is_requesting_static_resource(request_data.path.clone()) {
        let response_data = generate_static_response(request_data.clone());
        let response_data = add_some_headers(
            request_data,
            response_data,
            server_config.clone(),
            nonce.clone(),
        );
        return response_data;
    }

    let request_start = Instant::now();
    let cookies = request_data.cookies.clone();
    let (session, created_new_session) =
        match session::get_existing_or_make_new_session(db_pool.clone(), cookies).await {
            Ok((s, cns)) => (s, cns),
            Err(_) => {
                return HttpResponseData {
                    set_cookie: None,
                    status_code: StatusCode::Code500InternalServerError,
                    body: Some(b"500".to_vec()),
                    headers: {
                        let mut headers = http_response::Headers::new();
                        headers.insert(
                            "content-type",
                            http_stuff::http_response::ContentType::TextPlain.to_string(),
                        );
                        headers.insert("charset", Charset::Utf8.to_string());
                        headers.insert(
                            "connection",
                            http_stuff::http_response::KeepAliveOrClose::KeepAlive.to_string(),
                        );
                        headers
                    },
                };
            }
        };
    let notifications =
        session::read_and_delete_session_notifications(db_pool.clone(), session.id.clone()).await;
    let notifications = match notifications {
        Ok(ns) => ns,
        Err(_) => {
            return internal_server_error_500();
        }
    };
    let base_template_data = html_templating::html_template_structs::BaseTemplateData {
        title: "".to_string(),
        page: request_data.path.clone(),
        cloudflare_turnstile_enabled: server_config.cloudflare_turnstile_enabled,
        cloudflare_turnstile_site_key: server_config.cloudflare_turnstile_site_key.clone(),
        notifications: into_vec(notifications),
        nonce: nonce.clone(),
    };
    let response_data = route(
        db_pool.clone(),
        server_config.clone(),
        session.clone(),
        created_new_session,
        request_data.clone(),
        base_template_data,
    )
    .await
    .unwrap_or(internal_server_error_500());

    let response_data = add_elapsed(response_data, session.clone(), request_start, db_pool.clone()).await;

    let r = add_some_headers(request_data.clone(), response_data, server_config, nonce);

   if request_data.method == "GET" {
        let _ = session::set_custom_data(db_pool.clone(), session.id.clone(), "previous_page".to_string(), request_data.path_and_query.clone()).await;
    }
    r
}

pub fn add_some_headers(
    request_data: HttpRequestData,
    mut response_data: HttpResponseData,
    server_config: Config,
    nonce: String,
) -> HttpResponseData {
    // KEEP_ALIVE_ENABLED
    if server_config.keep_alive_enabled {
        response_data
            .headers
            .insert("connection", KeepAliveOrClose::KeepAlive.to_string());
    } else {
        response_data
            .headers
            .insert("connection", KeepAliveOrClose::Close.to_string());
    }

    // HSTS
    // I guess this will disable sts whenever I run this without ssl, right?..
    response_data.headers.insert(
        "Strict-Transport-Security",
        if server_config.using_https {
            "max-age=99999999; includeSubDomains; preload"
        } else {
            "max-age=0"
        }
        .to_string(),
    );

    // Other sec headers

    response_data
        .headers
        .insert("X-Content-Type-Options", "nosniff".to_string());
    response_data
        .headers
        .insert("X-Frame-Options", "DENY".to_string());

    //Chrome gives errors on this for localhost, why? Not sure...
    response_data
        .headers
        .insert("Cross-Origin-Embedder-Policy", "require-corp".to_string());
    response_data
        .headers
        .insert("Cross-Origin-Resource-Policy", "same-origin".to_string());
    response_data
        .headers
        .insert("Cross-Origin-Opener-Policy", "same-origin".to_string());
    response_data
        .headers
        .insert("Referrer-Policy", "no-referrer".to_string());
    response_data.headers.insert(
        "Permissions-Policy",
        "camera=(),\
microphone=(),\
geolocation=(),\
payment=(),\
usb=(),\
bluetooth=()"
            .to_string(),
    );
    let mut script_src = String::new();
    let mut frame_src = String::new();
    // if on pages using cf turntile, include the url in the csp headers

    if server_config.cloudflare_turnstile_enabled {
        match request_data.path.as_str() {
            "/signup" | "/login" => {
                script_src = String::from("https://challenges.cloudflare.com");
                frame_src = String::from("https://challenges.cloudflare.com");
            }
            _ => {}
        }
    }

    response_data.headers.insert(
        "Content-Security-Policy",
        format!(
            "default-src 'none'; \
        script-src 'nonce-{}' 'strict-dynamic' {script_src}; \
        frame-src {frame_src}; \
        object-src 'none'; \
        base-uri 'none'; \
        frame-ancestors 'none'; \
        img-src 'self'; \
        style-src 'self'; \
        connect-src 'self'; \
        font-src 'self'; \
        form-action 'self';",
            nonce.clone()
        ),
    );
    response_data
}

pub async fn add_elapsed(
    mut response_data: HttpResponseData,
    Session { id: session_id, .. }: Session,
    request_start: Instant,
    db_pool: sqlx::PgPool,
) -> HttpResponseData {
    let session_data = match get_session_by_session_id(session_id.clone(), db_pool.clone()).await {
        Ok(Ok(session_data)) => session_data,
        Ok(Err(e)) => {
            dbg!("Could not get session, returning as is", e);
            return response_data;
        }
        Err(_e) => {
            dbg!("Could not get session, returning as is", _e);
            return response_data;
        }
    };
    let elapsed_ms = request_start.elapsed().as_millis();

    let content_type_html = 'show_time: {
        if let Some(content_type) = response_data.headers.get("content-type")
            && content_type.contains(&ContentType::TextHtml.to_string())
        {
            break 'show_time true;
        }
        false
    };
    if let Some(body) = &mut response_data.body
        && content_type_html
        && let Ok(mut html) = String::from_utf8(body.clone())
    {
        let previous_ms = session_data.previous_page_load_time_ms;

        let timing_html = format!(
            r#"<div>
Previous page loading took {previous_ms} ms<br>
Page loading took {elapsed_ms} ms<br>
</div>"#
        );

        html = html.replace("</body>", &format!("{}\n</body>", timing_html));
        *body = html.into_bytes();
    }
    let elapsed_ms: i64 = match i64::try_from(elapsed_ms) {
        Ok(v) => v,
        Err(_e) => {
            dbg!("Try_from failed, could not convert u128 into i64", _e);
            return response_data;
        }
    };
    if set_previous_page_load_time_ms(db_pool, session_id, elapsed_ms)
        .await
        .is_err()
    {
        dbg!("Could not edit previous_page_load_time_ms");
    }

    response_data
}

pub fn is_requesting_static_resource(path: String) -> bool {
    let static_paths = ["/stylesheet.css", "/favicon.ico", "/app.js"];
    static_paths.contains(&path.as_str())
}

//#[tracing::instrument]
pub fn generate_static_response(request_data: HttpRequestData) -> HttpResponseData {
    let HttpRequestData { method, path, .. } = request_data;
    assert!(is_requesting_static_resource(path.clone()));
    tracing::info!("Entering static routing for {method} {path}");

    match (method.as_str(), path.as_str()) {
        // static resources
        ("GET", "/stylesheet.css") => {
            routes::stylesheet::get().unwrap_or(internal_server_error_500())
        }
        ("GET", "/favicon.ico") => routes::favicon::get().unwrap_or(internal_server_error_500()),
        ("GET", "/app.js") => {
            let appjs =
                read_on_dev_embed_on_release::read_to_string_on_dev_embed_on_release("app.js");

            http_stuff::http_response::ok(
                appjs.into_bytes(),
                http_response::ContentType::TextJavascript,
                Some(Charset::Utf8),
            )
        }
        _ => unreachable!("Should not happen"),
    }
}
