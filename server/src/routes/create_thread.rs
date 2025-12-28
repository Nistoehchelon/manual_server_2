use forum::{Forum, get_forum_by_id};
use html_templating::html_template_structs;
use crate::http_request_parsing::Query;
use crate::http_response::ok_html;
use crate::redirect::to;
use session::{Session, add_session_notification};
use user;

pub fn get(session: Session, query: Query, db_pool: &db_manager::DbPool) -> String {
    let user = user::get_user_by_user_id(session.user_id.clone(), db_pool).unwrap();
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = get_forum_by_id(forum_id.clone());
    if !forum.is_some() {
        add_session_notification(session.id.clone(), format!("Forum not found"), db_pool);
        return to("/forum".to_string());
    }
    let forum = forum.unwrap();
    if forum.access_level > user.access_level {
        add_session_notification(session.id.clone(), format!("Forum not found"), db_pool);
        return to("/forum".to_string());
    }

    if !user.can(user::Action::CreateThread) {
        return to("/forum".to_string());
    }
    ok_html(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.clone(), session::read_and_delete_session_notifications(session.id.clone())
    ))
}

fn get_html(title: String, forum: Forum, notifications: Vec<session::SessionNotification>) -> String {
    let html = html_templating::render(
        html_template_structs::CreateThreadTemplate {
            main_info: html_template_structs::MainInfo {
                title, notifications,

            }, forum,
        }
    );

    return html;
}
