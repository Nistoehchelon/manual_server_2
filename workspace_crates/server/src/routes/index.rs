use html_templating::html_template_structs::{
    IndexLoggedInTemplate, IndexNotLoggedInTemplate, MainInfo,
};

use session::{Session, SessionNotification, read_and_delete_session_notifications};
use user::{User, get_user_by_user_id};

use crate::http_response::internal_server_error_500;

pub fn get(session: Session, db_pool: &db_manager::DbPool) -> String {
    let notifs = match read_and_delete_session_notifications(db_pool, session.id.clone()) {
        Ok(ns) => ns,
        Err(_) => return internal_server_error_500(),
    };
    if session.logged_in {
        let user = match get_user_by_user_id(session.user_public_inapp_id.clone(), db_pool) {
            Ok(Some(u)) => u,
            _ => return internal_server_error_500(),
        };

        return crate::http_response::ok_html(get_html(session.clone(), notifs, user));
    }
    crate::http_response::ok_html(get_html(session.clone(), notifs, User::default()))
}

fn get_html(session: Session, notifications: Vec<SessionNotification>, user: User) -> String {
    if session.logged_in {
        let index_template = IndexLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome back!"),
                notifications,
            },
            user,
        };
        html_templating::render(index_template)
    } else {
        let index_template = IndexNotLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome!"),
                notifications,
            },
        };
        html_templating::render(index_template)
    }
}
