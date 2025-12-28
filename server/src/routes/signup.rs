use crate::{http_request_parsing::parse_username_and_password_from_body, redirect::to};
use html_templating::html_template_structs::{self, MainInfo};
use session::{
    Session, add_session_notification, edit_session, read_and_delete_session_notifications,
};
use user::try_to_create_new_user;

pub fn get(session: Session, db_pool: &db_manager::DbPool) -> String {
    if session.logged_in {
        add_session_notification(
            session.id.clone(),
            "You are already logged in, so signup is unaccessible. Log out first?".to_string(),
            db_pool,
        );
        return to("/".to_string());
    }

    // Show username form
    crate::http_response::ok_html(get_signup_html(read_and_delete_session_notifications(
        session.id.clone(),
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
        add_session_notification(session.id.clone(), String::from("Invalid body"), db_pool);
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
    let access_level = 1;

    match try_to_create_new_user(username, password, access_level, db_pool) {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_id = user.id.clone();
            edit_session(modified_session);
        }
        Err(error_message) => {
            add_session_notification(session.id, error_message, db_pool);
            return to("/signup".to_string());
        }
    }

    to("/".to_string())
}
