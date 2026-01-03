use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::{
    http_request_parsing::parse_username_and_password_from_body,
    http_response::{HttpResponse, internal_server_error_500, ok_html, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::try_to_create_new_user;

pub fn get(session: Session, db_pool: &db_manager::DbPool) -> HttpResponse {
    if session.logged_in {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!(
                    "You are already logged in, so signup is unaccessible. Log out first?"
                ),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return redirect_to("/".to_string());
    }

    let notifs = match read_and_delete_session_notifications(db_pool, session.id) {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    // Show username form
    ok_html(get_signup_html(notifs))
}

fn get_signup_html(notifications: Vec<session::SessionNotification>) -> String {
    return html_templating::render(html_template_structs::SignupTemplate {
        main_info: MainInfo {
            title: format!("Signup"),
            notifications: into_vec(notifications),
        },
    });
}

pub fn post(session: Session, body: Option<Vec<u8>>, db_pool: &db_manager::DbPool) -> HttpResponse {
   let body = match body {
    Some(b)=>b,
    None=> return internal_server_error_500(),
  };
    if session.logged_in {
        return redirect_to("/".to_string());
    }

    let (username, password) = match parse_username_and_password_from_body(body.clone()) {
        Ok((u, p)) => (u, p),
        Err(_) => {
            let _ = add_session_notification(
                db_pool,
                SessionNotification {
                    message: format!("Invalid body"),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            );
            return redirect_to("/".to_string());
        }
    };

    let user_creation_result = match try_to_create_new_user(db_pool, username, password) {
        Ok(ucr) => ucr,
        Err(_) => return internal_server_error_500(),
    };
    match user_creation_result {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_public_inapp_id = user.public_inapp_id.clone();
            edit_session(db_pool, session.id.clone(), modified_session);
        }
        Err(e) => {
            let _ = add_session_notification(
                db_pool,
                SessionNotification {
                    session_id: session.id.clone(),
                    message: e.message,
                    notification_type: session::SessionNotificationType::Error,
                },
            );
            return redirect_to("/signup".to_string());
        }
    }

    redirect_to("/".to_string())
}
