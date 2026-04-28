use csrf_stuff::validate_token;
use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, BaseTemplateData};
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, redirect_to},
};
use session::{Session, SessionNotification, add_session_notification, edit_session};
use user::find_by_username_and_password;

use crate::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
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

    Ok(http_stuff::http_response::ok_html(get_html(
        router_data.base_template_data,
        csrf_token,
        router_data.server_config.min_length_user_password,
        router_data.server_config.max_length_user_password,
    )))
}

pub async fn post(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let hasher_config = router_data.server_config.password_hasher_config;
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
    let body_bytes = router_data.http_request_data.body_bytes.clone();
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
            return Ok(redirect_to(session.previous_get_page.clone()));
        }
    };
    if router_data.server_config.cloudflare_turnstile_enabled {
        let cf_turnstile_token = match body_params.get("cf-turnstile-response").cloned() {
            Some(t) => t,
            None => {
                return Err(InternalAppError::new(
                    "Could not retrieve cf_turnstile_token from request body",
                ));
            }
        };
        let client_ip = router_data.http_request_data.x_real_ip;

        if !cloudflare_turnstile::validate_turnstile(
            router_data
                .server_config
                .cloudflare_turnstile_secret_key
                .clone(),
            cf_turnstile_token,
            client_ip,
        )
        .await
        {
            add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    session_id: session.id.clone(),
                    message: String::from("Could not validate Turnstile Captcha"),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to(session.previous_get_page.clone()));
        }
    }

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

    let user_result =
        find_by_username_and_password(username, password, db_pool.clone(), hasher_config)
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

fn get_html(mut base_template: BaseTemplateData, csrf_token: String, min_length_user_password: usize, max_length_user_password: usize) -> String {
    base_template.title = "Login".to_string();
    html_templating::render(html_template_structs::LoginTemplate {
        csrf_token,
        main_info: base_template,
        username_length_min: user::MIN_LENGTH_USERNAME,
        username_length_max: user::MAX_LENGTH_USERNAME,
        password_length_min: min_length_user_password,
        password_length_max: max_length_user_password,
        
    })
}
