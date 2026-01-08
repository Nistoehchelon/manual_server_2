use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::http_response::{ResponseData, internal_server_error_500, redirect_to};
use session::Session;
use user;

pub async fn get(session: Session, db_pool: &sqlx::PgPool) -> ResponseData {
    if !session.logged_in {
        return redirect_to("/".to_string());
    }
    let user = match user::get_user_by_user_id(session.user_id.clone(), db_pool).await {
        Ok(Some(u)) => u,
        _ => return internal_server_error_500(),
    };
    let notifications =
        match session::read_and_delete_session_notifications(db_pool, session.id.clone()).await {
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
    http_stuff::http_response::ok_html(html)
}
