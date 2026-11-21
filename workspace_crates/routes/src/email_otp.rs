use email_otp::PendingUser;
use html_templating::html_template_structs::{self, BaseTemplateData};
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, ok_html, redirect_to},
};
use session::{
    SessionNotification, add_session_notification, add_session_notification_error,
    add_session_notification_success, edit_session,
};
use user::ValidatedUsername;
use user_creation::try_to_create_new_user;

use routing_structs::RouterData;

pub async fn get(
    router_data: RouterData,
) -> anyhow::Result<HttpResponseData> {
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
        .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
        return Ok(redirect_to("/".to_string()));
    }
    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
   .map_err(|e|anyhow::anyhow!("{e:#?}"))?;

    // Show username form
    Ok(ok_html(get_email_otp_html(
        router_data.base_template_data,
        csrf_token,
    )?))
}

fn get_email_otp_html(mut base_template_data: BaseTemplateData, csrf_token: String) -> anyhow::Result<String> {
    base_template_data.title = "OTP".to_string();
    html_templating::render(html_template_structs::EmailOTPTemplate {
        main_info: base_template_data,
        csrf_token,
    })
}

pub async fn post(
    router_data: RouterData,
) -> anyhow::Result<HttpResponseData> {
    let RouterData {
        session,
        db_pool,
        http_request_data,
        site_config,
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
                .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
                return Ok(redirect_to("/".to_string()));
            }
        };

    let otp = body_params.get("otp").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (otp, csrf_token) = match (otp, csrf_token) {
        (Some(otp), Some(c)) => (otp, c),
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
            .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
            return Ok(redirect_to("/".to_string()));
        }
    };

    if site_config.cloudflare_turnstile_enabled {
        let cf_turnstile_token = match body_params.get("cf-turnstile-response").cloned() {
            Some(t) => t,
            None => {
                return Err(anyhow::anyhow!(
                    "Could not retrieve cf_turnstile_token from request body",
                ));
            }
        };
        let client_ip = http_request_data.x_real_ip;

        if !cloudflare_turnstile::validate_turnstile(
            router_data
                .site_config
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
            .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
            return Ok(redirect_to("/signup".to_string()));
        }
    }

    let csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
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
        .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
        return Ok(redirect_to("/signup".to_string()));
    }

    let mut tx = db_pool.begin().await.map_err(|e|anyhow::anyhow!("{e:#?}"))?;
    let validation = email_otp::consume_email_otp(&mut tx, otp.as_str()).await;
    let validation = match validation {
        Ok(v) => v,
        Err(e) => {
            add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    message: "Erm".to_string(),
                    session_id: session.id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
            tracing::error!("{e:#?}");
            return Ok(redirect_to("/email_otp".to_string()));
        }
    };
    let PendingUser {
        email,
        username,
        password_hash,
    } = validation;
    let Ok(username) = ValidatedUsername::validate(&mut tx, username).await else {
        add_session_notification_error(db_pool.clone(), session.id.clone(), "Oops".to_string())
            .await;
        return Ok(redirect_to("/".to_string()));
    };
    // add email, username, and password into pending table, create otp.
    // redirect user to /email_otp.
    let user_creation_result = try_to_create_new_user(&mut tx, username, password_hash)
        .await
        .map_err(|e|anyhow::anyhow!("{e:#?}"))?;

    match user_creation_result {
        Ok(user) => {
            user_creation_success(&session, &db_pool, site_config, &mut tx, email, user)
                .await
                .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
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
            .map_err(|e|anyhow::anyhow!("{e:#?}"))?;
            return Ok(redirect_to("/signup".to_string()));
        }
    }
    tx.commit().await.map_err(|e|anyhow::anyhow!("{e:#?}"))?;
    Ok(redirect_to("/".to_string()))
}

#[tracing::instrument(skip_all)]
async fn user_creation_success(
    session: &session::Session,
    db_pool: &sqlx::Pool<sqlx::Postgres>,
    site_config: site_config::SiteConfig,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    email: String,
    user: user::User,
) -> anyhow::Result<()> {
    let mut modified_session = session.clone();
    modified_session.logged_in = true;
    modified_session.user_id = user.id.clone();
    edit_session(db_pool.clone(), session.id.clone(), modified_session).await?;
    add_session_notification_success(
        db_pool.clone(),
        session.id.clone(),
        "Congratulations! You have been successfully registered.".to_string(),
    )
    .await;
    user::set_user_data(tx, &user.id, "email", email.as_str()).await?;
    tracing::info!("User with id {} was registered.", user.id.clone());
    tracing::info!("Sending data over to the auth server");
    let auth_req = auth_system_structs::internal_api::AddOrUpdateUserRequest {
        user_id: user.id.clone(),
        username: user.username.clone(),
        password_hash: user.password_hash.clone(),
        subscription_expires_at_timestamp: 0,
    };
    match auth_system_structs::internal_api::send_add_or_update_user_request(
        site_config.auth_server_url.clone(),
        site_config.auth_server_secret_prefix,
        site_config.auth_server_token,
        auth_req,
    )
    .await
    {
        Ok(s) => {
            tracing::debug!("auth server success: {s:#?}");
        }
        Err(e) => {
            tracing::debug!("auth server error: {e:#?}");
        }
    }
    Ok(())
}
