use db_manager;
use db_manager::DbPool;
use env_reader::get_env;
use server::http_request_parsing;
use server::redirect::to;
use server::routes;
use session;
use session::Session;
use session::SessionNotification;
use session::add_session_notification;
use storage;
use server::thread_pool;
use user::ADMIN_LVL;
use user::USER_LVL;
use user::get_user_by_user_id;

use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn main() {
    let env = get_env();
    let db_pool = db_manager::get_pool(env.database_str);

    let listener = TcpListener::bind(env.ip_and_port.clone()).expect("Failed to bind to address");
    println!("Server listening on http://{}", env.ip_and_port.clone());
    storage::initialize_storage_functionality();
    let thread_pool = thread_pool::ThreadPool::new(8);
    for stream in listener.incoming() {
        let db_pool = db_pool.clone();
        let stream = stream.unwrap();

        thread_pool.execute(move || {
            handle_connection(stream, &db_pool);
        });
    }

    println!("Shutting down server.");
}

fn handle_connection(mut stream: TcpStream, db_pool: &DbPool) {

    let request_data = match http_request_parsing::parse_request_data(&mut stream) {
        Some(data) => data,
        None => return,
    };

    let http_request_parsing::RequestData {
        method,
        path,
        body,
        cookies,
        query,
        path_with_query,
    } = request_data;

    let mut will_set_cookie = false;
    let session: session::Session = {
        let session_id: String =
            match cookies.get(storage::get_name_of_cookie_containing_session_id().as_str()) {
                Some(session_id) => {
                    if !session::session_with_specified_id_exists(session_id.clone(), &db_pool) {
                        will_set_cookie = true;
                        session::generate_session_id()
                    } else {
                        session_id.to_string()
                    }
                }
                None => {
                    will_set_cookie = true;
                    session::generate_session_id()
                }
            };
        if session::session_with_specified_id_exists(session_id.clone(), db_pool) {
            session::get_session_by_session_id(session_id.clone(), db_pool).unwrap()
        } else {
            session::create_new_session_with_id(session_id.clone(), db_pool)
        }
    };

    // Route handling
    let response = 'response_block: {
        if will_set_cookie {
            break 'response_block format!(
                "HTTP/1.1 302 Found\r\n\
                Set-Cookie: {}={}; Path=/\r\n\
                Location: {path_with_query}\r\n\
                Content-Type: text/html\r\n\r\n",
                storage::get_name_of_cookie_containing_session_id(),
                session.id.clone()
            )
            .into_bytes();
        }
        if !check_if_user_can_access_path(session.clone(), path.clone()) {
            add_session_notification(db_pool,
               SessionNotification {notification_type: session::SessionNotificationType::Error, session_id: session.id.clone(),
                message: format!("Page {path} is inaccessible at this particular nanosecond *sigh*"),}
            );
            break 'response_block to("/".to_string()).into_bytes();
        }
        if session.logged_in {
            if !get_user_by_user_id(session.user_id.clone(), db_pool).is_some() {
                let mut modified_session = session.clone();
                modified_session.logged_in = false;
                modified_session.user_id = "".to_string();
                session::edit_session(db_pool, modified_session.id.clone(), modified_session);
                break 'response_block to("/".to_string()).into_bytes();
            }
        }
        match (method.as_str(), path.as_str()) {
            ("GET", "/signup") => routes::signup::get(session,db_pool).into_bytes(),
            ("POST", "/signup") => routes::signup::post(session, body.clone(), db_pool).into_bytes(),
            ("GET", "/dashboard") => routes::dashboard::get(session, db_pool,).into_bytes(),
            ("GET", "/logout") => routes::logout::get(db_pool, session).into_bytes(),
            ("GET", "/login") => routes::login::get(db_pool, session).into_bytes(),
            ("POST", "/login") => routes::login::post(session, body.clone(), db_pool).into_bytes(),
            ("GET", "/forum") => routes::forum::get(session, query, db_pool).into_bytes(),
            ("GET", "/create_thread") => routes::create_thread::get(session, query, db_pool).into_bytes(),

            // static resources
            ("GET", "/stylesheet.css") => routes::stylesheet::get().into_bytes(),
            ("GET", "/favicon.ico") => routes::favicon::get(),
            _ => routes::index::get(session, db_pool).into_bytes(),
        }
    };

    stream.write_all(&response).ok();
}

fn check_if_user_can_access_path(session: Session, path: String) -> bool {
    let mut access_level = 0;
    if session.logged_in == true {
        access_level = USER_LVL;
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
        USER_LVL => {
            allowed = user_allowed;
        }
        ADMIN_LVL => {
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
