use crate::{
    http_request_parsing::Query,
    http_response::{self, internal_server_error_500},
    redirect::to,
};
use forum::{Forum, ForumThread, get_forum_by_id};
use html_templating::html_template_structs::{self, MainInfo};
use session::{
    Session, SessionNotification, add_session_notification, read_and_delete_session_notifications,
};
use user::get_user_by_user_id;

pub fn get(session: Session, query: Query, db_pool: &db_manager::DbPool) -> String {
    let forum_id = 'forum_id: {
        for kv in query.clone() {
            if kv.0 == "id" {
                break 'forum_id kv.1;
            }
        }
        "0".to_string()
    };
    let forum_lookup_result = match get_forum_by_id(forum_id.clone()) {
        Ok(opt) => opt,
        Err(_) => return internal_server_error_500(),
    };
    if forum_lookup_result.is_none() {
        let _ = add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Forum with id {} could not be found", forum_id),
                notification_type: session::SessionNotificationType::Error,
            },
        );
        return to("/forum".to_string());
    }
    let forum = forum_lookup_result.unwrap();
    let _user = match get_user_by_user_id(session.user_public_inapp_id.clone(), db_pool) {
        Ok(Some(u)) => u,
        _ => return internal_server_error_500(),
    };

    let forum_threads = match forum::get_forum_threads(forum.id.clone()) {
        Ok(ft) => ft,
        Err(_) => return internal_server_error_500(),
    };
    let title = format!("Forum \"{}\"", forum.name);
    let notifications = match read_and_delete_session_notifications(db_pool, session.id.clone()) {
        Ok(notifs) => notifs,
        _ => return internal_server_error_500(),
    };
    let html = get_html(title, true, true, forum, forum_threads, notifications);
    http_response::ok_html(html)
}

fn get_html(
    title: String,
    user_can_create_child_forum: bool,
    user_can_create_thread: bool,
    forum: Forum,
    forum_threads: Vec<ForumThread>,
    notifications: Vec<SessionNotification>,
) -> String {
    let html = html_templating::render(html_template_structs::ForumTemplate {
        main_info: MainInfo {
            title,
            notifications,
        },
        user_can_create_child_forum,
        user_can_create_thread,
        forum,
        forum_threads,
    });
    return html;
}
