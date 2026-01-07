use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::parse_username_and_password_from_body,
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
    let notifs = match read_and_delete_session_notifications(db_pool, session.public_id.clone()).await {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    http_stuff::http_response::ok_html(get_html(notifs))
}

pub async fn post(session: Session, body: Vec<u8>, db_pool: &sqlx::PgPool) -> HttpResponse {
    if session.logged_in {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.public_id.clone(),
                message: format!("You are already logged in."),
                notification_type: session::SessionNotificationType::Information,
            },
        );
        return redirect_to(String::from("/"));
    }

    let (username, password) = match parse_username_and_password_from_body(body) {
        Err(_) => {
            let _ = add_session_notification(
                db_pool,
                SessionNotification {
                    message: format!("Could not extract username and password from body"),
                    session_id: session.public_id.clone(),
                    notification_type: session::SessionNotificationType::Error,
                },
            );
            return redirect_to("/login".to_string());
        }
        Ok((u, p)) => (u, p),
    };

    let user_result = match get_by_username_and_password(username, password, db_pool).await {
        Ok(opt) => opt,
        _ => return internal_server_error_500(),
    };

    if user_result.is_none() {
        let _result = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.public_id.clone(),
                message: format!("Username and/or password incorrect"),
                notification_type: session::SessionNotificationType::Error,
            },
        ).await;
        return redirect_to("/login".to_string());
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_public_id = user.public_id.clone();
    session.logged_in = true;
    let _result = edit_session(db_pool, session.public_id.clone(), session.clone()).await;

    let _result = add_session_notification(
        db_pool,
        SessionNotification {
            session_id: session.clone().public_id,
            message: format!("Successfully logged in! Welcome!"),
            notification_type: session::SessionNotificationType::Success,
        },
    ).await;
    redirect_to("/".to_string())
}

fn get_html(notifications: Vec<SessionNotification>) -> String {
    let html = html_templating::render(html_template_structs::LoginTemplate {
        main_info: html_template_structs::MainInfo {
            title: format!("Login"),
            notifications: into_vec(notifications),
        },
    });
    return html;
}
