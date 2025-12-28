use crate::{
    http_request_parsing::Query,
    http_response,
    redirect::to,
   
};
use  forum::{Forum, ForumThread, get_forum_by_id};
use  session::{Session, add_session_notification};
   use user::{User, get_user_by_user_id};


pub fn get(session: Session, query: Query) -> String {
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
        );
        return to("/forum".to_string());
    }
    let forum = forum_lookup_result.unwrap();
    let user = get_user_by_user_id(session.user_id.clone()).unwrap();
    if forum.access_level > user.access_level {
        // sneaky message, tehee
        add_session_notification(
            session.id.clone(),
            format!("Forum with id {} could not be found", forum_id),
        );
        return to("/forum".to_string());
    }
    let forum_threads = forum::get_forum_threads(forum.id.clone());
    let title = "Forum".to_string();
    let html = get_html(title, user, forum, forum_threads);
    http_response::ok(html)
}

fn get_html(_title: String, user: User, forum: Forum, forum_threads: Vec<ForumThread>) -> String {
    let mut main_content = format!("");
    if user.can(user::Action::CreateForum) {
        let forum_id = forum.id.clone();
        let add = format!(
            r##"
        <a href="/create_forum?parent_id={forum_id}">Create a forum</a><br>
        "##
        );
        main_content = format!("{}{}", add, main_content);
    }
    if user.can(user::Action::CreateThread) {
        let forum_id = forum.id.clone();
        let add = format!(
            r##"
        <a href="/create_thread?parent_id={forum_id}">New thread</a><br>
        "##
        );
        main_content = format!("{}{}", add, main_content);
    }

    if forum_threads.len() == 0 {
        main_content = format!("{main_content}<br>This forum is currently empty.");
    }

   main_content
}
