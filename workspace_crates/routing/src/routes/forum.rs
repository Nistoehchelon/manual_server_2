use forum::{Forum, ForumThread, get_forum_by_id};
use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::{
    http_request_parsing::Query,
    http_response::{self, HttpResponse, internal_server_error_500, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, read_and_delete_session_notifications,
};
use user::get_user_by_user_id;

pub async fn get(session: Session, query: Query, db_pool: &sqlx::PgPool) -> HttpResponse {
    let forum_id = query.clone().iter().find(|(k, _)| k.as_str() == "id").map(|(_, v)| v.clone()).unwrap_or_else(|| "0".to_string());

    let forum = match get_forum_by_id(db_pool, forum_id.clone()).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            let _ = add_session_notification(
                db_pool,
                SessionNotification {
                    session_id: session.id.clone(),
                    message: format!("Forum with id {} could not be found", forum_id),
                    notification_type: session::SessionNotificationType::Error,
                },
            );
            return redirect_to("/forum".to_string());
        }
        Err(_) => return internal_server_error_500(),
    };

    let _user = match get_user_by_user_id(session.user_id.clone(), db_pool).await {
        Ok(Some(u)) => u,
        _ => return internal_server_error_500(),
    };

    let forum_threads = match forum::get_forum_threads(db_pool, forum.id.clone()).await {
        Ok(ft) => ft,
        Err(_) => return internal_server_error_500(),
    };
    let subforums = match forum::get_subforums(db_pool, forum.id.clone()).await {
        Ok(ss) => ss,
        Err(_) => return internal_server_error_500(),
    };

    let title = forum.name.clone();
    let notifications = match read_and_delete_session_notifications(db_pool, session.id.clone()).await {
        Ok(notifs) => notifs,
        _ => return internal_server_error_500(),
    };
    let html = get_html(
        title,
        true,
        true,
        forum,
        forum_threads,
        subforums,
        notifications,
    );
    http_response::ok_html(html)
}

fn get_html(
    title: String,
    user_can_create_child_forum: bool,
    user_can_create_thread: bool,
    forum: Forum,
    forum_threads: Vec<ForumThread>,
    subforums: Vec<Forum>,
    notifications: Vec<SessionNotification>,
) -> String {
    let html = html_templating::render(html_template_structs::ForumTemplate {
        main_info: MainInfo {
            title,
            notifications: into_vec(notifications),
        },
        user_can_create_child_forum,
        user_can_create_thread,
        forum: forum.into(),
        subforums: into_vec(subforums),
        forum_threads: into_vec(forum_threads),
    });
    return html;
}
