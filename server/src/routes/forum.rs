use crate::{
    http_request_parsing::Query,
    http_response,
    redirect::to,
   
};
use  forum::{Forum, ForumThread, get_forum_by_id};
use html_templating::html_template_structs::{self, MainInfo};
use  session::{Session, SessionNotification, add_session_notification, read_and_delete_session_notifications};
   use user::{User, get_user_by_user_id};


pub fn get(session: Session, query: Query, db_pool: &db_manager::DbPool) -> String {
    let forum_id = 'forum_id: {
        for kv in query.clone() {
            if kv.0 == "id" {
                break 'forum_id kv.1;
            }
        }
        "0".to_string()
    };
    let forum_lookup_result = get_forum_by_id(forum_id.clone());
    if forum_lookup_result.is_none() {
        add_session_notification(
            session.id.clone(),
            format!("Forum with id {} could not be found", forum_id),
            db_pool,
        );
        return to("/forum".to_string());
    }
    let forum = forum_lookup_result.unwrap();
    let user = get_user_by_user_id(session.user_id.clone(), db_pool).unwrap();
    if forum.access_level > user.access_level {
        // sneaky message, tehee
        add_session_notification(
            session.id.clone(),
            format!("Forum with id {} could not be found", forum_id),
            db_pool
        );
        return to("/forum".to_string());
    }
    let forum_threads = forum::get_forum_threads(forum.id.clone());
    let title = format!("Forum \"{}\"", forum.name);
    let notifications = read_and_delete_session_notifications(session.id.clone());
    let html = get_html(title, user, forum, forum_threads, notifications);
    http_response::ok_html(html)
}

fn get_html(title: String, user: User, forum: Forum, forum_threads: Vec<ForumThread>, notifications: Vec<SessionNotification>) -> String { 
    let html = html_templating::render(
        html_template_structs::ForumTemplate {
            main_info: MainInfo {
                title,
                notifications,
            },
            user,
            forum,
            forum_threads,
        }
    );
    return html;
}
