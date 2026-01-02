use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::parse_username_and_password_from_body,
    http_response::internal_server_error_500, http_response::redirect_to,
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::get_by_username_and_password;

pub fn get(db_pool: &db_manager::DbPool, session: Session) -> String {
    if session.logged_in {
        return redirect_to("/".to_string());
    }
    let notifs = match read_and_delete_session_notifications(db_pool, session.id.clone()) {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    http_stuff::http_response::ok_html(get_html(notifs))
}

pub fn post(session: Session, body: Option<Vec<u8>>, db_pool: &db_manager::DbPool) -> String {

  let body = match body {
    Some(b)=>b,
    None=> return internal_server_error_500(),
  };
    if session.logged_in {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("You are already logged in."),
                notification_type: session::SessionNotificationType::Information,
            },
        );
        return redirect_to(String::from("/"));
    }
    
    let (username, password) = match parse_username_and_password_from_body(body) {
        Err(_) => {
                    let _ = add_session_notification(
            db_pool,
            SessionNotification {
                message: format!("Could not extract username and password from body"),
                session_id: session.id.clone(),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return redirect_to("/login".to_string());
        },
        Ok((u, p)) => (u, p),
    };
    
    let user_result = match get_by_username_and_password(username, password, db_pool) {
        Ok(opt) => opt,
        _ => return internal_server_error_500(),
    };

    if user_result.is_none() {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Username and/or password incorrect"),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return redirect_to("/login".to_string());
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_public_inapp_id = user.public_inapp_id.clone();
    session.logged_in = true;
    edit_session(db_pool, session.id.clone(), session.clone());

    let _ = add_session_notification(
        db_pool,
        SessionNotification {
            session_id: session.clone().id,
            message: format!("Successfully logged in! Welcome!"),
            notification_type: session::SessionNotificationType::Success,
        },
    );
    redirect_to("/".to_string())
}

fn get_html(notifications: Vec<SessionNotification>) -> String {
    let html = html_templating::render(html_template_structs::LoginTemplate {
        main_info: html_template_structs::MainInfo {
            title: format!("Login"),
            notifications: into_vec(notifications),
        },
    });
    return html;
}
