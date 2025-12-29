use crate::{http_request_parsing::parse_username_and_password_from_body, redirect::to};
use html_templating::html_template_structs::{self, MainInfo};
use session::{
    Session, SessionNotification, add_session_notification, edit_session, read_and_delete_session_notifications
};
use user::try_to_create_new_user;

pub fn get(session: Session, db_pool: &db_manager::DbPool) -> String {
    if session.logged_in {
        add_session_notification(
db_pool, SessionNotification { session_id: session.id.clone(), message: format!("You are already logged in, so signup is unaccessible. Log out first?"), notification_type: session::SessionNotificationType::Error }
        );
        return to("/".to_string());
    }

    // Show username form
    crate::http_response::ok_html(get_signup_html(read_and_delete_session_notifications(
        db_pool, session.id.clone(),
    )))
}

fn get_signup_html(notifications: Vec<session::SessionNotification>) -> String {
    return html_templating::render(html_template_structs::SignupTemplate {
        main_info: MainInfo {
            title: format!("Signup"),
            notifications: notifications,
        },
    });
}

pub fn post(session: Session, body: String, db_pool: &db_manager::DbPool) -> String {
    if session.logged_in {
        return to("/".to_string());
    }

    let parse_result = parse_username_and_password_from_body(body.clone());
    if parse_result.is_err() {
        add_session_notification(db_pool, SessionNotification {message: format!("Invalid body"), session_id: session.id.clone(), notification_type: session::SessionNotificationType::Error});
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
    let access_level = 1;

    match try_to_create_new_user(username, password, access_level, db_pool) {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_id = user.id.clone();
            edit_session(db_pool, session.id.clone(), modified_session);
        }
        Err(error_message) => {
            add_session_notification(db_pool, SessionNotification {session_id: session.id.clone(), message: error_message, notification_type: session::SessionNotificationType::Error });
            return to("/signup".to_string());
        }
    }

    to("/".to_string())
}
