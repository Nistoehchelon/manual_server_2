use csrf_stuff::validate_token;
use error_structs::InternalAppError;
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::find_by_username_and_password;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        http_request_data: _,
    }: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;
    let notifs = read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
        .await
        .map_err(InternalAppError::new)?;
    Ok(http_stuff::http_response::ok_html(get_html(
        notifs, csrf_token,
    )))
}

pub async fn post(
    RouterData {
        db_pool,
        session,
        http_request_data: request_data,
    }: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    if session.logged_in {
        add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: "You are already logged in.".to_string(),
                notification_type: session::SessionNotificationType::Information,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to(String::from("/")));
    }
    let body_bytes = request_data.body_bytes.clone();
    let (username, password, csrf_token) =
        match http_request_parsing::parse_body_into_string_form_input_params(body_bytes) {
            Err(_) => {
                add_session_notification(
                    db_pool.clone(),
                    SessionNotification {
                        message: "Could not extract params from body".to_string(),
                        session_id: session.id.clone(),
                        notification_type: session::SessionNotificationType::Error,
                    },
                )
                .await
                .map_err(InternalAppError::new)?;
                return Ok(redirect_to("/login".to_string()));
            }
            Ok(params) => {
                let username_opt = params.get("username").cloned();
                let password_opt = params.get("password").cloned();
                let csrf_token_opt = params.get("csrf_token").cloned();
                match (username_opt, password_opt, csrf_token_opt) {
                    (Some(u), Some(p), Some(c)) => (u, p, c),
                    _ => {
                        add_session_notification(
                            db_pool.clone(),
                            SessionNotification {
                                message: "Could not extract params from body".to_string(),
                                session_id: session.id.clone(),
                                notification_type: session::SessionNotificationType::Error,
                            },
                        )
                        .await
                        .map_err(InternalAppError::new)?;
                        return Ok(redirect_to("/login".to_string()));
                    }
                }
            }
        };

    let csrf_token_valid = validate_token(db_pool.clone(), session.id.clone(), csrf_token)
        .await
        .map_err(InternalAppError::new)?;
    if !csrf_token_valid {
        add_session_notification(
            db_pool.clone(),
            SessionNotification {
                session_id: session.id.clone(),
                message: "Could not verify CSRF token".to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/login".to_string()));
    }
    let user_result = find_by_username_and_password(username, password, db_pool.clone())
        .await
        .map_err(InternalAppError::new)?;

    if user_result.is_none() {
        add_session_notification(
            db_pool.clone(),
            SessionNotification {
                session_id: session.id.clone(),
                message: "Username and/or password incorrect".to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/login".to_string()));
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_id = user.id.clone();
    session.logged_in = true;
    edit_session(db_pool.clone(), session.id.clone(), session.clone())
        .await
        .map_err(InternalAppError::new)?;

    add_session_notification(
        db_pool.clone(),
        SessionNotification {
            session_id: session.clone().id,
            message: "Successfully logged in! Welcome!".to_string(),
            notification_type: session::SessionNotificationType::Success,
        },
    )
    .await
    .map_err(InternalAppError::new)?;
    Ok(redirect_to("/".to_string()))
}

fn get_html(notifications: Vec<SessionNotification>, csrf_token: String) -> String {
    html_templating::render(html_template_structs::LoginTemplate {
        csrf_token,
        main_info: html_template_structs::BaseTemplateData {
            title: "Login".to_string(),
            notifications: into_vec(notifications),
        },
    })
}
