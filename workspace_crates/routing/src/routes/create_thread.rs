use abac::{CanCreateThreadInput, CanSetCustomThreadIdWhenCreatingItInput};
use error_structs::InternalAppError;
use forum;
use forum::find_forum_by_id;
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self},
    http_response::{HttpResponseData, bad_request_400, ok_html, redirect_to},
};
use session::{SessionNotification, add_session_notification};

use crate::RouterData;

pub async fn get(
    mut router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let query = router_data.http_request_data.query.clone();
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();

    let can_create_thread = abac::can_create_forum_thread(CanCreateThreadInput {
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

    let forum = find_forum_by_id(db_pool.clone(), forum_id.clone())
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

    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;

    let title = format!(r##"Create a thread in forum {}"##, forum.name.clone());
    router_data.base_template_data.title = title;

    let html = html_templating::render(html_template_structs::CreateThreadTemplate {
        main_info: router_data.base_template_data,
        forum: forum.into(),
        csrf_token,
        draft_thread_content: session::get_custom_data_or_empty_string(
            db_pool.clone(),
            session.id.clone(),
            "draft_forum_thread_content".to_string(),
        )
        .await
        .map_err(InternalAppError::new)?,

        draft_thread_name: session::get_custom_data_or_empty_string(
            db_pool.clone(),
            session.id.clone(),
            "draft_forum_thread_name".to_string(),
        )
        .await
        .map_err(InternalAppError::new)?,

        draft_thread_custom_id: {
            let draft = session::get_custom_data_or_empty_string(
                db_pool.clone(),
                session.id.clone(),
                "draft_forum_thread_custom_id".to_string(),
            )
            .await
            .map_err(InternalAppError::new)?;
            if draft.is_empty() {
                "[auto]".to_string()
            } else {
                draft
            }
        },

        can_set_custom_id: abac::can_set_custom_thread_id_when_creating_it(
            CanSetCustomThreadIdWhenCreatingItInput {
                db_pool: db_pool.clone(),
                session: session.clone(),
            },
        )
        .await
        .map_err(InternalAppError::new)?,
    });
    Ok(ok_html(html))
}

pub async fn post(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    let request_data = router_data.http_request_data.clone();
    let body_bytes = request_data.body_bytes.clone();
    let can_create_thread = abac::can_create_forum_thread(CanCreateThreadInput {
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
    let forum = find_forum_by_id(db_pool.clone(), forum_id.clone())
        .await
        .map_err(InternalAppError::new)?;

    session::set_custom_data(
        db_pool.clone(),
        session.id.clone(),
        "draft_forum_thread_name".to_string(),
        name.clone(),
    )
    .await
    .map_err(InternalAppError::new)?;
    session::set_custom_data(
        db_pool.clone(),
        session.id.clone(),
        "draft_forum_thread_content".to_string(),
        content.clone(),
    )
    .await
    .map_err(InternalAppError::new)?;
    if forum.is_none() {
        session::add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            "Could not find forum with specified id".to_string(),
        )
        .await
        .map_err(InternalAppError::new)?;
        return Ok(redirect_to(format!("/create_thread?forum_id={forum_id}")));
    }


    let mut thread_id = forum::generate_unique_thread_id(db_pool.clone())
        .await
        .map_err(InternalAppError::new)?;
    
    if let Some(custom_id) = params.get("custom_id")
        && custom_id != "[auto]"
    {
        let custom_id = custom_id.to_owned();
        if !abac::can_set_custom_thread_id_when_creating_it(
            CanSetCustomThreadIdWhenCreatingItInput {
                db_pool: db_pool.clone(),
                session: session.clone(),
            },
        )
        .await
        .map_err(InternalAppError::new)?
        {
            add_session_notification(
                db_pool,
                SessionNotification {
                    session_id: session.id.clone(),
                    message:
                        "You do not have access to `set thread id upon creation` functionality"
                            .to_string(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to(format!("create_thread?forum_id={forum_id}")));
        } else {
            session::set_custom_data(
                db_pool.clone(),
                session.id.clone(),
                "draft_forum_thread_custom_id".to_string(),
                custom_id.clone(),
            )
            .await
            .map_err(InternalAppError::new)?;
            let custom_id_available = forum::find_thread(db_pool.clone(), custom_id.clone())
                .await
                .map_err(InternalAppError::new)?
                .is_none();
            if !custom_id_available {
                add_session_notification(
                    db_pool,
                    SessionNotification {
                        session_id: session.id.clone(),
                        message: "Custom id already taken by some other thread".to_string(),
                        notification_type: session::SessionNotificationType::Error,
                    },
                )
                .await
                .map_err(InternalAppError::new)?;

                return Ok(redirect_to(format!("/create_thread?forum_id={forum_id}")));
            } else {
                thread_id = custom_id;
            }
        }
    }

    let thread_id = forum::create_thread(
        db_pool.clone(),
        thread_id,
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
