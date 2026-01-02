use db_manager::DbPool;
use http_stuff::{
    http_request_parsing::HttpRequestData,
    http_response::{internal_server_error_500, redirect_to},
};
use session::{self, Session, SessionNotification, add_session_notification};
use user::{self, get_user_by_user_id};

use crate::routes;

pub fn route(
    db_pool: &db_manager::DbPool,
    session: Session,
    created_new_session: bool,
    request_data: HttpRequestData,
) -> Vec<u8> {
    let HttpRequestData {
        path_with_query,
        path,
        method,
        query,
        body_bytes,
        ..
    } = request_data;
    let body_string = String::from_utf8_lossy(&body_bytes.clone()).to_string();
    let response = 'response_block: {
        if created_new_session {
            break 'response_block format!(
                "HTTP/1.1 302 Found\r\n\
                Set-Cookie: {}={}; Path=/\r\n\
                Location: {path_with_query}\r\n\
                Content-Type: text/html\r\n\r\n",
                session::get_name_of_cookie_containing_session_id(),
                session.id.clone()
            )
            .into_bytes();
        }
        if !check_if_user_can_access_path(session.clone(), path.clone()) {
            match add_session_notification(
                db_pool,
                SessionNotification {
                    notification_type: session::SessionNotificationType::Error,
                    session_id: session.id.clone(),
                    message: format!(
                        "Page {path} is inaccessible at this particular nanosecond *sigh*"
                    ),
                },
            ) {
                Ok(_) => {}
                Err(_) => return internal_server_error_500().into_bytes(),
            }
            break 'response_block redirect_to("/".to_string()).into_bytes();
        }
        if session.logged_in {
            let user_associated_with_session_actually_exists =
                match get_user_by_user_id(session.user_public_inapp_id.clone(), db_pool) {
                    Ok(Some(_)) => true,
                    Ok(None) => false,
                    Err(_) => return internal_server_error_500().into_bytes(),
                };
            if !user_associated_with_session_actually_exists {
                let mut modified_session = session.clone();
                modified_session.logged_in = false;
                modified_session.user_public_inapp_id = "".to_string();
                session::edit_session(db_pool, modified_session.id.clone(), modified_session);
                break 'response_block redirect_to("/".to_string()).into_bytes();
            }
        }
        dbg!(format!("Entering routing for {method} {path}"));
        match (method.as_str(), path.as_str()) {
            ("GET", "/signup") => routes::signup::get(session, db_pool).into_bytes(),
            ("POST", "/signup") => {
                routes::signup::post(session, body_string, db_pool).into_bytes()
            }
            ("GET", "/dashboard") => routes::dashboard::get(session, db_pool).into_bytes(),
            ("GET", "/logout") => routes::logout::get(db_pool, session).into_bytes(),
            ("GET", "/login") => routes::login::get(db_pool, session).into_bytes(),
            ("POST", "/login") => routes::login::post(session, body_string, db_pool).into_bytes(),
            ("GET", "/forum") => routes::forum::get(session, query, db_pool).into_bytes(),
            ("GET", "/create_thread") => {
                routes::create_thread::get(session, query, db_pool).into_bytes()
            }

            _ => routes::index::get(session, db_pool).into_bytes(),
        }
    };
    return response;
}

fn check_if_user_can_access_path(session: Session, path: String) -> bool {
    let mut access_level = 0;
    if session.logged_in == true {
        access_level = 1;
    }
    let mut allowed: Vec<&str> = vec![];
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
            allowed = vec!["/signup", "/login", "/", "/stylesheet.css", "/favicon.ico"];
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

pub fn generate_response(request_data: HttpRequestData, db_pool: &DbPool) -> Vec<u8> {
    let cookies = request_data.cookies.clone();
    let (session, created_new_session) =
        match session::get_existing_or_make_new_session(db_pool, cookies) {
            Ok((s, cns)) => (s, cns),
            Err(_) => return internal_server_error_500().into_bytes(),
        };
    // Route handling
    let response = route(db_pool, session, created_new_session, request_data);

    return response;
}

pub fn is_requesting_static_resource(path: String) -> bool {
    let static_paths = vec!["/stylesheet.css", "/favicon.ico"];
    return static_paths.contains(&path.as_str());
}

pub fn generate_static_response(request_data: HttpRequestData) -> Vec<u8> {
    let HttpRequestData { method, path, .. } = request_data;
    assert!(is_requesting_static_resource(path.clone()));
    dbg!(format!("Entering static routing for {method} {path}"));
    let response = match (method.as_str(), path.as_str()) {
        // static resources
        ("GET", "/stylesheet.css") => routes::stylesheet::get().into_bytes(),
        ("GET", "/favicon.ico") => routes::favicon::get(),
        _ => unreachable!("Should not happen"),
    };

    return response;
}
