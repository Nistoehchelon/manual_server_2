use csrf_stuff::validate_token;
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponse, internal_server_error_500, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, edit_session,
    read_and_delete_session_notifications,
};
use user::get_by_username_and_password;

pub async fn get(db_pool: &sqlx::PgPool, session: Session) -> HttpResponse {
    if session.logged_in {
        return redirect_to("/".to_string());
    }
    let csrf_token = match csrf_stuff::get_or_create_token(db_pool, session.id.clone(), csrf_stuff::DesignatedPurpose::Normal).await {
        Ok(ct) => ct,
        Err(_)=> {
            return internal_server_error_500();
        }
    };
    let notifs = match read_and_delete_session_notifications(db_pool, session.id.clone()).await {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    http_stuff::http_response::ok_html(get_html(notifs, csrf_token))
}

pub async fn post(session: Session, body: Vec<u8>, db_pool: &sqlx::PgPool) -> HttpResponse {
    if session.logged_in {
        add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("You are already logged in."),
                notification_type: session::SessionNotificationType::Information,
            },
        )
        .await;
        return redirect_to(String::from("/"));
    }

    let (username, password, csrf_token) =
        match http_request_parsing::parse_body_into_string_form_input_params(body) {
            Err(_) => {
                add_session_notification(
                    db_pool,
                    SessionNotification {
                        message: format!("Could not extract params from body"),
                        session_id: session.id.clone(),
                        notification_type: session::SessionNotificationType::Error,
                    },
                )
                .await;
                return redirect_to("/login".to_string());
            }
            Ok(params) => {
                let username_opt = params.get("username").cloned();
                let password_opt = params.get("password").cloned();
                let csrf_token_opt = params.get("csrf_token").cloned();
                match (username_opt, password_opt, csrf_token_opt) {
                    (Some(u), Some(p), Some(c)) => (u, p, c),
                    _ => {
                        add_session_notification(
                            db_pool,
                            SessionNotification {
                                message: format!(
                                    "Could not extract params from body"
                                ),
                                session_id: session.id.clone(),
                                notification_type: session::SessionNotificationType::Error,
                            },
                        )
                        .await;
                        return redirect_to("/login".to_string());
                    }
                }
            }
        };

    let csrf_token_valid = match validate_token(db_pool, session.id.clone(), csrf_token).await {
        Ok(b) => b,
        Err(_) => {
            return internal_server_error_500();
        },
    };
    if !csrf_token_valid {
        add_session_notification(db_pool, SessionNotification { session_id: session.id.clone(), message: format!("Could not verify CSRF token"), notification_type: session::SessionNotificationType::Error }).await;
        return redirect_to(format!("/login"));
    }
    let user_result = match get_by_username_and_password(username, password, db_pool).await {
        Ok(opt) => opt,
        _ => return internal_server_error_500(),
    };

    if user_result.is_none() {
        add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Username and/or password incorrect"),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await;
        return redirect_to("/login".to_string());
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_id = user.id.clone();
    session.logged_in = true;
    match edit_session(db_pool, session.id.clone(), session.clone()).await {
        Err(_) => {
            return internal_server_error_500();
        }
        _ => {}
    };

    add_session_notification(
        db_pool,
        SessionNotification {
            session_id: session.clone().id,
            message: format!("Successfully logged in! Welcome!"),
            notification_type: session::SessionNotificationType::Success,
        },
    )
    .await;
    redirect_to("/".to_string())
}

fn get_html(notifications: Vec<SessionNotification>, csrf_token: String,) -> String {
    let html = html_templating::render(html_template_structs::LoginTemplate {
        csrf_token,
        main_info: html_template_structs::MainInfo {
            title: format!("Login"),
            notifications: into_vec(notifications),
        },
    });
    return html;
}
