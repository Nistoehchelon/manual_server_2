use html_templating::html_template_structs::{
    self, IndexLoggedInTemplate, IndexNotLoggedInTemplate, MainInfo,
};

use into_vec::into_vec;
use session::{Session, SessionNotification, read_and_delete_session_notifications};
use user::{User, get_user_by_user_id};

use http_stuff::http_response::{HttpResponse, internal_server_error_500};

pub async fn get(session: Session, db_pool: &sqlx::PgPool) -> HttpResponse {
    let notifs = match read_and_delete_session_notifications(db_pool, session.public_id.clone()).await {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    if session.logged_in {
        let user = match get_user_by_user_id(session.user_public_id.clone(), db_pool).await {
            Ok(Some(u)) => u,
            _ => return internal_server_error_500(),
        };

        return http_stuff::http_response::ok_html(get_html(session.clone(), notifs, user));
    }
    http_stuff::http_response::ok_html(get_html(session.clone(), notifs, User::default()))
}

fn get_html(session: Session, notifications: Vec<SessionNotification>, user: User) -> String {
    if session.logged_in {
        let index_template = IndexLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome back!"),
                notifications: into_vec(notifications),
            },
            user: html_template_structs::User {
                username: user.username.clone(),
            },
        };
        html_templating::render(index_template)
    } else {
        let index_template = IndexNotLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome!"),
                notifications: into_vec(notifications),
            },
        };
        html_templating::render(index_template)
    }
}
