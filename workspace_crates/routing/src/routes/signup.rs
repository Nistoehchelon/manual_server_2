use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::{
    http_request_parsing::{self},
    http_response::{ResponseData, ok_html, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::try_to_create_new_user;

pub async fn get(session: Session, db_pool: &sqlx::PgPool) -> Result<ResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!(
                    "You are already logged in, so signup is unaccessible. Log out first?"
                ),
                notification_type: session::SessionNotificationType::Error,
            },
        ).await.map_err(InternalAppError::new)?;
        return Ok(redirect_to("/".to_string()));
    }
    let csrf_token = csrf_stuff::get_or_create_token(db_pool, session.id.clone(), csrf_stuff::DesignatedPurpose::Normal).await.map_err(InternalAppError::new)?;

    let notifs = read_and_delete_session_notifications(db_pool, session.id).await.map_err(InternalAppError::new)?;
    // Show username form
    Ok(ok_html(get_signup_html(notifs, csrf_token)))
}

fn get_signup_html(notifications: Vec<session::SessionNotification>, csrf_token: String,) -> String {
    return html_templating::render(html_template_structs::SignupTemplate {
        main_info: MainInfo {
            title: format!("Signup"),
            notifications: into_vec(notifications),
        },
        csrf_token,
    });
}

pub async fn post(session: Session, body: Vec<u8>, db_pool: &sqlx::PgPool) -> Result<ResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }

    let body_params = match http_request_parsing::parse_body_into_string_form_input_params(body) {
        Ok(bp)=> bp,
        Err(_) => {
           add_session_notification(
                db_pool,
                SessionNotification {
                    message: format!("Invalid body"),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            ).await.map_err(InternalAppError::new)?;
            return Ok(redirect_to("/".to_string()));
        }
    };
let username = body_params.get("username").cloned();
let password = body_params.get("password").cloned();
let csrf_token = body_params.get("csrf_token").cloned();
let (username, password, csrf_token) = match (username, password, csrf_token) {
    (Some(u), Some(p), Some(c)) => {
        (u, p, c)
    },
    _ => {
      add_session_notification(
                db_pool,
                SessionNotification {
                    message: format!("Invalid body"),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            ).await.map_err(InternalAppError::new)?;
            return Ok(redirect_to("/signup".to_string()));  
    }
};

let csrf_token_valid = csrf_stuff::validate_token(db_pool, session.id.clone(), csrf_token.clone()).await.map_err(InternalAppError::new)?;
if !csrf_token_valid {
        add_session_notification(
                db_pool,
                SessionNotification {
                    message: format!("csrf_token validation error"),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            ).await.map_err(InternalAppError::new)?;
            return Ok(redirect_to("/signup".to_string()));  
}

    let user_creation_result = try_to_create_new_user(db_pool, username, password).await.map_err(InternalAppError::new)?;
    match user_creation_result {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_id = user.id.clone();
             edit_session(db_pool, session.id.clone(), modified_session).await.map_err(InternalAppError::new)?;
        }
        Err(e) => {
            add_session_notification(
                db_pool,
                SessionNotification {
                    session_id: session.id.clone(),
                    message: e.message,
                    notification_type: session::SessionNotificationType::Error,
                },
            ).await.map_err(InternalAppError::new)?;
            return Ok(redirect_to("/signup".to_string()));
        }
    }

    Ok(redirect_to("/".to_string()))
}
