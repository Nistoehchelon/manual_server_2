use abac::CanCreateThreadInput;
use error_structs::InternalAppError;
use forum;
use forum::{Forum, get_forum_by_id};
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self},
    http_response::{ResponseData, bad_request_400, ok_html, redirect_to},
};
use session::{SessionNotification, add_session_notification};

use crate::RouterData;

pub async fn get(
    // session: Session,
    // query: Query,
    // db_pool: sqlx::PgPool,
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    let query = request_data.query.clone();

    let can_create_thread = abac::can_create_thread(CanCreateThreadInput {
        db_pool: db_pool.clone(),
        session_id: session.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?;
    if !can_create_thread {
        return Ok(redirect_to("/".to_string()));
    }
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = get_forum_by_id(db_pool.clone(), forum_id.clone())
        .await
        .map_err(InternalAppError::new)?;

    if forum.is_none() {
        add_session_notification(
            db_pool.clone(),
            SessionNotification {
                session_id: session.id.clone(),
                message: "Forum not found".to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/forum".to_string()));
    }
    let forum = forum.unwrap();
    let notifs =
        session::read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
            .await
            .map_err(InternalAppError::new)?;

    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;

    Ok(ok_html(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.clone(),
        notifs,
        csrf_token,
    )))
}

fn get_html(
    title: String,
    forum: Forum,
    notifications: Vec<session::SessionNotification>,
    csrf_token: String,
) -> String {
    

    html_templating::render(html_template_structs::CreateThreadTemplate {
        main_info: html_template_structs::MainInfo {
            title,
            notifications: into_vec::into_vec(notifications),
        },
        forum: forum.into(),
        csrf_token,
    })
}

pub async fn post(
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    let body_bytes = request_data.body_bytes.clone();
    let can_create_thread = abac::can_create_thread(CanCreateThreadInput {
        db_pool: db_pool.clone(),
        session_id: session.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?;
    if !can_create_thread {
        return Ok(redirect_to("/".to_string()));
    }
    let params = http_request_parsing::parse_body_into_string_form_input_params(body_bytes)
        .map_err(InternalAppError::new)?;
    dbg!(params.clone());

    let name = params.get("name").cloned();
    let content = params.get("content").cloned();
    let forum_id = params.get("forum_id").cloned();
    let csrf_token = params.get("csrf_token").cloned();

    let (name, content, forum_id, csrf_token) = match (name, content, forum_id, csrf_token) {
        (Some(n), Some(c), Some(fi), Some(ct)) => (n, c, fi, ct),
        _ => return Ok(bad_request_400()),
    };
    let is_csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !is_csrf_token_valid {
        return Ok(bad_request_400());
    }

    let thread_id = forum::create_thread(
        db_pool.clone(),
        name,
        content,
        session.user_id.clone(),
        forum_id.clone(),
    )
    .await
    .map_err(InternalAppError::new)?;

    add_session_notification(
        db_pool.clone(),
        SessionNotification {
            session_id: session.id.clone(),
            message: format!("Creation of thread with id {thread_id} successful."),
            notification_type: session::SessionNotificationType::Success,
        },
    )
    .await
    .map_err(InternalAppError::new)?;
    Ok(redirect_to(format!(
        "/forum_thread?id={}",
        thread_id.clone()
    )))
}
