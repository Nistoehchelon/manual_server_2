use error_structs::InternalAppError;
use forum::get_forum_by_id;
use html_templating::html_template_structs::{self};
use http_stuff::http_response::{self, HttpResponseData, redirect_to};
use into_vec::into_vec;
use session::{
    SessionNotification, add_session_notification,
};
use user::find_user_by_user_id;

use crate::RouterData;

pub async fn get(
    mut router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let http_request_data = router_data.http_request_data.clone();
    let query = http_request_data.query.clone();
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
   
    let mut user_can_create_forum = false;
    let mut user_can_create_thread = false;
    if user_opt.is_some() {
        user_can_create_thread = abac::can_create_forum_thread(abac::CanCreateThreadInput {
            db_pool: db_pool.clone(),
            session_id: session.id.clone(),
        })
        .await
        .map_err(InternalAppError::new)?;
        user_can_create_forum = abac::can_create_forum(abac::CanCreateForumInput {
            db_pool: db_pool.clone(),
            session_id: session.id.clone(),
        })
        .await
        .map_err(InternalAppError::new)?;
    }

    router_data.base_template_data.title = title;

    let html = html_templating::render(html_template_structs::ForumTemplate {
        main_info: router_data.base_template_data,
        user_can_create_forum,
        user_can_create_thread,
        forum: forum.into(),
        subforums: into_vec(subforums),
        forum_threads: into_vec(forum_threads),
        forum_threads_pagination: vec![],
    });
    Ok(http_response::ok_html(html))
}
