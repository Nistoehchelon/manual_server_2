
use email_otp::CreateEmailOtpInput;
use error_structs::InternalAppError;
use html_templating::html_template_structs::{self, BaseTemplateData};
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, ok_html, redirect_to},
};
use session::{
    SessionNotification, add_session_notification, add_session_notification_error,
    add_session_notification_success,
};
use user::{ValidatedPassword, ValidatedUsername};

use crate::{RouterData, validate_turnstile};

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let RouterData {
        session, db_pool, ..
    } = router_data.clone();
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

    // Show username form
    Ok(ok_html(get_signup_html(
        router_data.base_template_data,
        csrf_token,
        router_data.server_config.min_length_user_password,
        router_data.server_config.max_length_user_password,
    )))
}

fn get_signup_html(
    mut base_template_data: BaseTemplateData,
    csrf_token: String,
    min_length_user_password: usize,
    max_length_user_password: usize,
) -> String {
    base_template_data.title = "Signup".to_string();
    html_templating::render(html_template_structs::SignupTemplate {
        main_info: base_template_data,
        csrf_token,
        username_length_min: user::MIN_LENGTH_USERNAME,
        username_length_max: user::MAX_LENGTH_USERNAME,
        password_length_min: min_length_user_password,
        password_length_max: max_length_user_password,
    })
}


pub async fn post(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let RouterData {
        session,
        db_pool,
        http_request_data,
        server_config,
        ..
    } = router_data.clone();
    if session.logged_in {
        return Ok(redirect_to("/".to_string()));
    }
    let body_bytes = http_request_data.body_bytes.clone();
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
    // TODO: email verification
    let email = body_params.get("email").cloned();
    let username = body_params.get("username").cloned();
    let password = body_params.get("password").cloned();
    let password_repeated = body_params.get("password_repeated").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (email, username, password, password_repeated, csrf_token) =
        match (email, username, password, password_repeated, csrf_token) {
            (Some(e), Some(u), Some(p), Some(pr), Some(c)) => (e, u, p, pr, c),
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

    if server_config.cloudflare_turnstile_enabled
        && !validate_turnstile(
            body_params.clone(),
            server_config
                .cloudflare_turnstile_post_parameter_name
                .clone(),
            http_request_data.x_real_ip,
            server_config.cloudflare_turnstile_secret_key.clone(),
        )
        .await
        .map_err(InternalAppError::new)?
    {
        add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            String::from("Could not validate Turnstile Captcha"),
        )
        .await;
        return Ok(redirect_to("/signup".to_string()));
    }

    let csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !csrf_token_valid {
        add_session_notification_error(db_pool.clone(), session.id.clone(), "csrf_token validation error".to_string())
        .await;
        return Ok(redirect_to("/signup".to_string()));
    }

    let mut tx = db_pool.begin().await.map_err(InternalAppError::new)?;
    let username = match ValidatedUsername::validate(&mut tx, username).await {
        Ok(u) => u,
        Err(e) => {
            add_session_notification_error(db_pool.clone(), session.id.clone(), e.message).await;
            return Ok(redirect_to(session.previous_get_page.clone()));
        }
    };
    let password = match ValidatedPassword::validate(
        password,
        router_data.server_config.min_length_user_password,
        router_data.server_config.max_length_user_password,
    ) {
        Ok(u) => u,
        Err(e) => {
            add_session_notification_error(db_pool.clone(), session.id.clone(), e.message).await;
            return Ok(redirect_to(session.previous_get_page.clone()));
        }
    };
    if password.to_string() != password_repeated {
        add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            "Passwords do not match.".to_string(),
        )
        .await;
        return Ok(redirect_to(session.previous_get_page.clone()));
    }

    let email_available = user::find_user_data_by_name_and_value(&mut tx, "email", email.as_str())
        .await
        .map_err(InternalAppError::new)?
        .is_empty();
    if !email_available {
        // I do not reveal this to the user who is trying to enter the email. Show same msg as on succcess.
        add_session_notification_success(
            db_pool.clone(),
            session.id.clone(),
            "Check your email".to_string(),
        )
        .await;
        return Ok(redirect_to("/email_otp".to_string()));
    }

    let password_hash = password_hashing::hash_password(
        password.to_string(),
        server_config.password_hasher_config.clone(),
    )
    .map_err(InternalAppError::new)?;

    email_otp::create_email_otp(
        db_pool.clone(),
        CreateEmailOtpInput {
            email,
            username: username.to_string(),
            password_hash,
        },
    )
    .await
    .map_err(InternalAppError::new)?;
    add_session_notification_success(
        db_pool.clone(),
        session.id.clone(),
        "Check your email".to_string(),
    )
    .await;
    Ok(redirect_to("/email_otp".to_string()))
}
