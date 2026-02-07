use std::time::Instant;

use error_structs::InternalAppError;
use http_stuff::{
    http_request_parsing::HttpRequestData,
    http_response::{
        Charset, ContentType, ResponseData, SetCookieHeader, StatusCode, internal_server_error_500,
        redirect_to, response, standalone_response,
    },
};
use session::{
    self, Session, SessionNotification, add_session_notification, get_session_by_session_id,
    set_previous_page_load_time_ms,
};
use user::{self, get_user_by_user_id};

use crate::routes;

#[derive(Clone)]
pub struct RouterData {
    pub db_pool: sqlx::PgPool,
    pub session: Session,
    pub request_data: HttpRequestData,
}

pub async fn route(
    db_pool: sqlx::PgPool,
    session: Session,
    created_new_session: bool,
    request_data: HttpRequestData,
) -> Result<ResponseData, InternalAppError> {
    let HttpRequestData {
        path_and_query: _path_and_query,
        path,
        method,
        query: _,
        body_bytes: _,
        ..
    } = request_data.clone();

    let mut response_data: ResponseData = 'response_block: {
        if !check_if_user_can_access_path(session.clone(), path.clone()) {
            add_session_notification(
                db_pool,
                SessionNotification {
                    notification_type: session::SessionNotificationType::Error,
                    session_id: session.id.clone(),
                    message: format!(
                        "Page {path} is inaccessible at this particular nanosecond *sigh*"
                    ),
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            break 'response_block redirect_to("/".to_string());
        }
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
        let router_data = RouterData {
            db_pool: db_pool.clone(),
            session: session.clone(),
            request_data: request_data.clone(),
        };
        dbg!(format!("Entering routing for {method} {path}"));
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
            }

            _ => routes::index::get(router_data.clone())
                .await
                .unwrap_or(internal_server_error_500()),
        }
    };

    if created_new_session {
        let ch = SetCookieHeader {
            key: session::get_name_of_cookie_containing_session_id(),
            value: session.id.clone(),
            path: "/".to_string(),
        };
        match response_data.set_cookie_headers {
            None => {
                let mut v = vec![];
                v.push(ch);
                response_data.set_cookie_headers = Some(v);
            }
            Some(ref mut sch) => {
                sch.push(ch);
            }
        }
    }

    Ok(response_data)
}

fn check_if_user_can_access_path(session: Session, path: String) -> bool {
    let mut access_level = 0;
    if session.logged_in {
        access_level = 1;
    }

    let mut allowed: Vec<&str> = vec![];

    let guest_allowed = vec![
        "/signup",
        "/login",
        "/",
        "/stylesheet.css",
        "/favicon.ico",
        "/forum",
        "/forum_thread",
        "/forum_thread_message_versions",
    ];

    let user_allowed = vec![
        "/logout",
        "/dashboard",
        "/",
        "/forum",
        "/stylesheet.css",
        "/favicon.ico",
        "/create_thread",
        "/forum_thread",
        "/forum_thread_message_versions",
    ];
    let mut admin_allowed: Vec<&str> = user_allowed.clone();
    admin_allowed.extend(vec!["/create_forum"]);

    match access_level {
        0 => {
            allowed = guest_allowed;
        }
        1 => {
            allowed = user_allowed;
        }
        4 => {
            allowed = admin_allowed;
        }
        _ => {
            unreachable!();
        }
    }
    allowed.contains(&path.as_str())
}

pub async fn response_to_bytes(response_data: ResponseData) -> Vec<u8> {
    
    match response(response_data) {
        Ok(rb) => rb,
        Err(_) => {
            standalone_response(StatusCode::Code500InternalServerError, "Error".to_string())
        }
    }
}
pub async fn generate_response_data(
    request_data: HttpRequestData,
    db_pool: sqlx::PgPool,
) -> ResponseData {
    if is_requesting_static_resource(request_data.path.clone()) {
        let response_data = generate_static_response(request_data);
        return response_data;
    }

    let request_start = Instant::now();
    let cookies = request_data.cookies.clone();
    let (session, created_new_session) =
        match session::get_existing_or_make_new_session(db_pool.clone(), cookies).await {
            Ok((s, cns)) => (s, cns),
            Err(_) => {
                return ResponseData {
                    status_code: StatusCode::Code500InternalServerError,
                    content_type: http_stuff::http_response::ContentType::TextPlain,
                    charset: Some(Charset::Utf8),
                    body: Some(b"500".to_vec()),
                    set_cookie_headers: None,
                    location: None,
                    connection: http_stuff::http_response::KeepAliveOrClose::KeepAlive,
                };
            }
        };
    // Route handling
    let response_data = route(
        db_pool.clone(),
        session.clone(),
        created_new_session,
        request_data,
    )
    .await
    .unwrap_or(internal_server_error_500());
    

    add_elapsed(response_data, session, request_start, db_pool).await
}

pub async fn add_elapsed(
    mut response_data: ResponseData,
    Session { id: session_id, .. }: Session,
    request_start: Instant,
    db_pool: sqlx::PgPool,
) -> ResponseData {
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
    if let Some(body) = &mut response_data.body
        && response_data.content_type == ContentType::TextHtml
            && let Ok(mut html) = String::from_utf8(body.clone()) {
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
    if let Err(_) = set_previous_page_load_time_ms(db_pool, session_id, elapsed_ms).await {
        dbg!("Could not edit previous_page_load_time_ms");
    }
    response_data
}

pub fn is_requesting_static_resource(path: String) -> bool {
    let static_paths = ["/stylesheet.css", "/favicon.ico"];
    static_paths.contains(&path.as_str())
}

pub fn generate_static_response(request_data: HttpRequestData) -> ResponseData {
    let HttpRequestData { method, path, .. } = request_data;
    assert!(is_requesting_static_resource(path.clone()));
    dbg!(format!("Entering static routing for {method} {path}"));
    

    match (method.as_str(), path.as_str()) {
        // static resources
        ("GET", "/stylesheet.css") => {
            routes::stylesheet::get().unwrap_or(internal_server_error_500())
        }
        ("GET", "/favicon.ico") => routes::favicon::get().unwrap_or(internal_server_error_500()),
        _ => unreachable!("Should not happen"),
    }
}
