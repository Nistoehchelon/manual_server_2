use crate::{http_response::internal_server_error_500, redirect::to};
use html_templating::html_template_structs::{self, MainInfo};
use session::Session;
use user;

pub fn get(session: Session, db_pool: &db_manager::DbPool) -> String {
    if !session.logged_in {
        return to("/".to_string());
    }
    let user = match user::get_user_by_user_id(session.user_public_inapp_id.clone(), db_pool) {
        Ok(Some(u)) => u,
        _ => return internal_server_error_500(),
    };
    let notifications =
        match session::read_and_delete_session_notifications(db_pool, session.id.clone()) {
            Ok(n) => n,
            _ => return internal_server_error_500(),
        };
    let html = html_templating::render(html_template_structs::DashboardTemplate {
        main_info: MainInfo {
            title: format!("Your Dashboard"),
            notifications: into_vec::into_vec(notifications),
        },
        username: user.username.clone(),
    });
    crate::http_response::ok_html(html)
}
