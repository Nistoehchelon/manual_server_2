use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, BaseTemplateData};
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, ok_html, redirect_to},
};
use into_vec::into_vec;
use session::{
    SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::try_to_create_new_user;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        http_request_data: _,
    }: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        add_session_notification(
            db_pool.clone(),
            SessionNotification {
                session_id: session.id.clone(),
                message: "You are already logged in, so signup is unaccessible. Log out first?"
                    .to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/".to_string()));
    }
    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;

    let notifs = read_and_delete_session_notifications(db_pool.clone(), session.id)
        .await
        .map_err(InternalAppError::new)?;
    // Show username form
    Ok(ok_html(get_signup_html(notifs, csrf_token)))
}

fn get_signup_html(notifications: Vec<session::SessionNotification>, csrf_token: String) -> String {
    html_templating::render(html_template_structs::SignupTemplate {
        main_info: BaseTemplateData {
            title: "Signup".to_string(),
            notifications: into_vec(notifications),
        },
        csrf_token,
    })
}

pub async fn post(
    RouterData {
        db_pool,
        session,
        http_request_data: request_data,
    }: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let body_bytes = request_data.body_bytes.clone();
    let body_params =
        match http_request_parsing::parse_body_into_string_form_input_params(body_bytes) {
            Ok(bp) => bp,
            Err(_) => {
                add_session_notification(
                    db_pool.clone(),
                    SessionNotification {
                        message: "Invalid body".to_string(),
                        session_id: session.id.clone(),
                        notification_type: session::SessionNotificationType::Error,
                    },
                )
                .await
                .map_err(InternalAppError::new)?;
                return Ok(redirect_to("/".to_string()));
            }
        };
    let username = body_params.get("username").cloned();
    let password = body_params.get("password").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();
    let (username, password, csrf_token) = match (username, password, csrf_token) {
        (Some(u), Some(p), Some(c)) => (u, p, c),
        _ => {
            add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    message: "Invalid body".to_string(),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/signup".to_string()));
        }
    };

    let csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !csrf_token_valid {
        add_session_notification(
            db_pool.clone(),
            SessionNotification {
                message: "csrf_token validation error".to_string(),
                session_id: session.id.clone(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/signup".to_string()));
    }

    let user_creation_result = try_to_create_new_user(db_pool.clone(), username, password)
        .await
        .map_err(InternalAppError::new)?;
    match user_creation_result {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_id = user.id.clone();
            edit_session(db_pool.clone(), session.id.clone(), modified_session)
                .await
                .map_err(InternalAppError::new)?;
        }
        Err(e) => {
            add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    session_id: session.id.clone(),
                    message: e.message,
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/signup".to_string()));
        }
    }

    Ok(redirect_to("/".to_string()))
}
