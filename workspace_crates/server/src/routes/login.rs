use crate::{
    http_request_parsing::parse_username_and_password_from_body,
    http_response::internal_server_error_500,
    redirect::{self, to},
};
use html_templating::html_template_structs;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::get_by_username_and_password;

pub fn get(db_pool: &db_manager::DbPool, session: Session) -> String {
    if session.logged_in {
        return redirect::to("/".to_string());
    }
    let notifs = match read_and_delete_session_notifications(db_pool, session.id.clone()) {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    crate::http_response::ok_html(get_html(notifs))
}

pub fn post(session: Session, body: String, db_pool: &db_manager::DbPool) -> String {
    if session.logged_in {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("You are already logged in."),
                notification_type: session::SessionNotificationType::Information,
            },
        );
        return to(String::from("/"));
    }
    let parse_result = parse_username_and_password_from_body(body);
    if parse_result.is_err() {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                message: parse_result.unwrap_err(),
                session_id: session.id.clone(),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
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
        return to("/login".to_string());
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
    redirect::to("/".to_string())
}

fn get_html(notifications: Vec<SessionNotification>) -> String {
    let html = html_templating::render(html_template_structs::LoginTemplate {
        main_info: html_template_structs::MainInfo {
            title: format!("Login"),
            notifications,
        },
    });
    return html;
}
