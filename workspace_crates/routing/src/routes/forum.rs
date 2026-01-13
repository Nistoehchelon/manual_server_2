use error_structs::InternalAppError;
use forum::{Forum, ForumThread, get_forum_by_id};
use html_templating::html_template_structs::{self, MainInfo};
use http_stuff::{
    http_request_parsing::Query,
    http_response::{self, ResponseData, redirect_to},
};
use into_vec::into_vec;
use session::{
    Session, SessionNotification, add_session_notification, read_and_delete_session_notifications,
};
use user::find_user_by_user_id;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData
) -> Result<ResponseData, error_structs::InternalAppError> {
    let query = request_data.query.clone();
    let forum_id = query
        .clone()
        .iter()
        .find(|(k, _)| k.as_str() == "id")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "0".to_string());

    let forum = match get_forum_by_id(db_pool.clone(), forum_id.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        Some(f) => f,
        None => {
            add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    session_id: session.id.clone(),
                    message: format!("Forum with id {} could not be found", forum_id),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/forum".to_string()));
        }
    };

    let user_opt = find_user_by_user_id(session.user_id.clone(), db_pool.clone())
        .await
        .map_err(InternalAppError::new)?;

    let forum_threads = forum::get_forum_threads(db_pool.clone(), forum.id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let subforums = forum::get_subforums(db_pool.clone(), forum.id.clone())
        .await
        .map_err(InternalAppError::new)?;

    let title = forum.name.clone();
    let notifications = read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let mut user_can_create_forum = false;
    let mut user_can_create_thread = false;
    if user_opt.is_some() {
        user_can_create_thread = true;
        user_can_create_forum = abac::can_create_forum(abac::CanCreateForumInput {
            db_pool: db_pool.clone(),
            session_id: session.id.clone(),
        })
        .await
        .map_err(InternalAppError::new)?;
    }

    let html = get_html(GetHtmlArgs {
        title,
        user_can_create_forum,
        user_can_create_thread,
        forum,
        forum_threads,
        subforums,
        notifications,
    });
    Ok(http_response::ok_html(html))
}

struct GetHtmlArgs {
    pub title: String,
    pub user_can_create_forum: bool,
    pub user_can_create_thread: bool,
    pub forum: Forum,
    pub forum_threads: Vec<ForumThread>,
    pub subforums: Vec<Forum>,
    pub notifications: Vec<SessionNotification>,
}
fn get_html(
    GetHtmlArgs {
        title,
        user_can_create_forum,
        user_can_create_thread,
        forum,
        forum_threads,
        subforums,
        notifications,
    }: GetHtmlArgs,
) -> String {
    let html = html_templating::render(html_template_structs::ForumTemplate {
        main_info: MainInfo {
            title,
            notifications: into_vec(notifications),
        },
        user_can_create_forum,
        user_can_create_thread,
        forum: forum.into(),
        subforums: into_vec(subforums),
        forum_threads: into_vec(forum_threads),
        forum_threads_pagination: vec![],
    });
    return html;
}
