use crate::http_response::ok_html;
use crate::redirect::to;
use crate::{http_request_parsing::Query, http_response::internal_server_error_500};
use forum::{Forum, get_forum_by_id};
use html_templating::html_template_structs;
use session::{Session, SessionNotification, add_session_notification};
use user;

pub fn get(session: Session, query: Query, db_pool: &db_manager::DbPool) -> String {
    let _user = user::get_user_by_user_id(session.user_public_inapp_id.clone(), db_pool).unwrap();
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = match get_forum_by_id(db_pool, forum_id.clone()) {
        Ok(opt) => opt,
        Err(_) => return internal_server_error_500(),
    };

    if !forum.is_some() {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Forum not found"),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return to("/forum".to_string());
    }
    let forum = forum.unwrap();
    let notifs = match session::read_and_delete_session_notifications(db_pool, session.id.clone()) {
        Ok(n) => n,
        Err(_) => return internal_server_error_500(),
    };

    ok_html(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.clone(),
        notifs,
    ))
}

fn get_html(
    title: String,
    forum: Forum,
    notifications: Vec<session::SessionNotification>,
) -> String {
    let html = html_templating::render(html_template_structs::CreateThreadTemplate {
        main_info: html_template_structs::MainInfo {
            title,
            notifications: into_vec::into_vec(notifications),
        },
        forum: forum.into(),
    });

    return html;
}
