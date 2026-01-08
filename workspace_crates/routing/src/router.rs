
use http_stuff::{
    http_request_parsing::HttpRequestData,
    http_response::{ResponseData, SetCookieHeader, StatusCode, internal_server_error_500, redirect_to, response, standalone_response},
};
use session::{self, Session, SessionNotification, add_session_notification};
use user::{self, get_user_by_user_id};

use crate::routes;

pub async fn route(
    db_pool: &sqlx::PgPool,
    session: Session,
    created_new_session: bool,
    request_data: HttpRequestData,
) -> ResponseData {
    let HttpRequestData {
        path_and_query: _path_and_query,
        path,
        method,
        query,
        body_bytes,
        ..
    } = request_data;

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
            ).await;
            break 'response_block redirect_to("/".to_string());
        }
        if session.logged_in {
            let user_associated_with_session_actually_exists =
                match get_user_by_user_id(session.user_id.clone(), db_pool).await {
                    Ok(Some(_)) => true,
                    Ok(None) => false,
                    Err(_) => return internal_server_error_500(),
                };
            if !user_associated_with_session_actually_exists {
                let mut modified_session = session.clone();
                modified_session.logged_in = false;
                modified_session.user_id = "".to_string();
                match session::edit_session(db_pool, modified_session.id.clone(), modified_session).await {
                    Err(_) => {
                        return internal_server_error_500();
                    }, 
                    _ => {},
                };
                break 'response_block redirect_to("/".to_string());
            } 
        }
        dbg!(format!("Entering routing for {method} {path}"));
        match (method.as_str(), path.as_str()) {
            ("GET", "/signup") => routes::signup::get(session.clone(), db_pool).await,
            ("POST", "/signup") => routes::signup::post(session.clone(), body_bytes, db_pool).await,
            ("GET", "/dashboard") => routes::dashboard::get(session.clone(), db_pool).await,
            ("GET", "/logout") => routes::logout::get(db_pool, session.clone()).await,
            ("GET", "/login") => routes::login::get(db_pool, session.clone()).await,
            ("POST", "/login") => routes::login::post(session.clone(), body_bytes, db_pool).await,
            ("GET", "/forum") => routes::forum::get(session.clone(), query, db_pool).await,
            ("GET", "/create_thread") => routes::create_thread::get(session.clone(), query, db_pool).await,
            ("POST", "/create_thread") => routes::create_thread::post(session.clone(), body_bytes, db_pool),

            _ => routes::index::get(session.clone(), db_pool).await,
        }
    };

     if created_new_session {
        let ch = SetCookieHeader { value: format!("{}={}; Path=/", session::get_name_of_cookie_containing_session_id(),
                session.id.clone()) };
        match response_data.set_cookie_headers {
            None => {
                let mut v = vec![];
                v.push(ch);
                response_data.set_cookie_headers = Some(v);
            },
            Some(ref mut sch) => {
                sch.push(ch);
            }
        }
     }
     
    return response_data;
}

fn check_if_user_can_access_path(session: Session, path: String) -> bool {
    let mut access_level = 0;
    if session.logged_in == true {
        access_level = 1;
    }

    let mut allowed: Vec<&str> = vec![];

    let guest_allowed = vec!["/signup", "/login", "/", "/stylesheet.css", "/favicon.ico", "/forum", "/thread"];

    let user_allowed = vec![
        "/logout",
        "/dashboard",
        "/",
        "/forum",
        "/stylesheet.css",
        "/favicon.ico",
        "/create_thread",
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
    if allowed.contains(&path.as_str()) {
        return true;
    } else {
        return false;
    }
}

pub async fn generate_response(request_data: HttpRequestData, db_pool: &sqlx::PgPool) -> Vec<u8> {
    if is_requesting_static_resource(request_data.path.clone()) {
        let response_data = generate_static_response(request_data);
        let response_bytes = match response(response_data) {
            Ok(rb) => rb,
            Err(_) => {
                return standalone_response(StatusCode::InternalServerError500, format!("Error"));
            }
        };
        return response_bytes;
    }

    let cookies = request_data.cookies.clone();
    let (session, created_new_session) =
        match session::get_existing_or_make_new_session(db_pool, cookies).await {
            Ok((s, cns)) => (s, cns),
            Err(_) => return standalone_response(StatusCode::InternalServerError500, format!("internal_server_error_500")),
        };
    // Route handling
    let response_data = route(db_pool, session, created_new_session, request_data).await;
let response_bytes = match response(response_data) {
        Ok(rb) => rb,
        Err(_) => {
            standalone_response(StatusCode::InternalServerError500, format!(":("))
        }
     };
    return response_bytes;
}

pub fn is_requesting_static_resource(path: String) -> bool {
    let static_paths = vec!["/stylesheet.css", "/favicon.ico"];
    return static_paths.contains(&path.as_str());
}

pub fn generate_static_response(request_data: HttpRequestData) -> ResponseData {
    let HttpRequestData { method, path, .. } = request_data;
    assert!(is_requesting_static_resource(path.clone()));
    dbg!(format!("Entering static routing for {method} {path}"));
    let response = match (method.as_str(), path.as_str()) {
        // static resources
        ("GET", "/stylesheet.css") => routes::stylesheet::get(),
        ("GET", "/favicon.ico") => routes::favicon::get(),
        _ => unreachable!("Should not happen"),
    };

    return response;
}
