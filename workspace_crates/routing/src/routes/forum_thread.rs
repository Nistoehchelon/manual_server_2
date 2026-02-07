use error_structs::{self, InternalAppError};
use html_templating;
use html_templating::html_template_structs;
use html_templating::html_template_structs::Paginator;
use http_stuff::http_request_parsing;
use http_stuff::http_response::bad_request_400;
use http_stuff::http_response::ok_html;
use http_stuff::http_response::{ResponseData, redirect_to};
use into_vec::into_vec;
use pagination::pagination::Pagination;
use session::add_session_notification;
use session::read_and_delete_session_notifications;
use url::Url;
use user::find_user_by_user_id;

use crate::RouterData;

pub const MESSAGES_PER_PAGE: u64 = 15;

pub async fn get(
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    let query = request_data.query.clone();
    let thread_id = query.get("id").cloned();
    let thread_id = match thread_id {
        Some(id) => id,
        None => {
            add_session_notification(
                db_pool.clone(),
                session::SessionNotification {
                    session_id: session.id.clone(),
                    message: "Could not find id in query string".to_string(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/forum".to_string()));
        }
    };
    let thread_page = query.get("page");
    let thread_page = match thread_page {
        Some(p) => {
            let mut temp = p.parse::<u64>().map_err(InternalAppError::new)?;
            if temp == 0 {
                temp = 1
            }
            temp
        }
        None => 1,
    };

    let thread = match forum::find_thread(db_pool.clone(), thread_id.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        Some(t) => t,
        None => {
            add_session_notification(
                db_pool.clone(),
                session::SessionNotification {
                    session_id: session.id.clone(),
                    message: "Could not find thread with specified id".to_string(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/forum".to_string()));
        }
    };

    let _user_opt = find_user_by_user_id(session.user_id.clone(), db_pool.clone())
        .await
        .map_err(InternalAppError::new)?;

    let limit_per_page = MESSAGES_PER_PAGE;
    let thread_message_ids = forum::get_thread_message_ids(
        db_pool.clone(),
        thread_id.clone(),
        thread_page,
        limit_per_page,
    )
    .await
    .map_err(InternalAppError::new)?;

    let thread_messages_count =
        forum::get_count_thread_messages(db_pool.clone(), thread_id.clone())
            .await
            .map_err(InternalAppError::new)?;
    let current_url = Url::new(request_data.path.clone(), request_data.query.clone());
    let pagi = Pagination {
        records_count: thread_messages_count,
        records_per_page: limit_per_page,
        current_page: thread_page,
        current_url: current_url.clone(),
        page_param: "page".to_string(),
    };
    let thread_pagination_elements = pagi.get_pagination_elements();

    let mut thread_messages: Vec<
        html_templating::html_template_structs::ForumThreadMessageCombinedData,
    > = Vec::with_capacity(usize::try_from(limit_per_page).map_err(InternalAppError::new)?);
    for msg_id in thread_message_ids {
        let latest_message_version =
            forum::get_latest_message_version(db_pool.clone(), msg_id.clone())
                .await
                .map_err(InternalAppError::new)?;
        let content = latest_message_version.content;

        let initial_message_version =
            forum::get_initial_message_version(db_pool.clone(), msg_id.clone())
                .await
                .map_err(InternalAppError::new)?;
        let user_id = initial_message_version.user_id;
        let username = user::get_user_by_user_id(db_pool.clone(), user_id.clone())
            .await
            .map_err(InternalAppError::new)?
            .username;

        // TODO: convert to string
        let created_at = String::from("todo created_at");

        let modifications_count = forum::get_modifications_count(db_pool.clone(), msg_id.clone())
            .await
            .map_err(InternalAppError::new)?;

        let tm = html_templating::html_template_structs::ForumThreadMessageCombinedData {
            id: msg_id,
            content,
            created_at,
            user_id,
            user_username: username,
            modifications_count,
        };
        thread_messages.push(tm);
    }

    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;

    let html = html_templating::render(html_template_structs::ForumThread {
        main_info: html_template_structs::MainInfo {
            title: String::new(),
            notifications: into_vec(
                read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
                    .await
                    .map_err(InternalAppError::new)?,
            ),
        },
        thread_messages,
        thread_name: thread.name,
        current_url: current_url.clone().to_string(),
        thread_page,
        forum_id: thread.forum_id,
        csrf_token,
        thread_paginator: Paginator {
            name: "thread_paginator".to_string(),
            elements: thread_pagination_elements,
        },
        user_can_add_message: abac::can_add_forum_thread_message(session.clone())
            .await
            .map_err(InternalAppError::new)?,
    });

    Ok(ok_html(html))
}

pub async fn post(
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    let query = request_data.query.clone();
    let body_bytes = request_data.body_bytes.clone();
    let current_url = Url::new(request_data.path.clone(), query.clone());

    let body_params = http_request_parsing::parse_body_into_string_form_input_params(body_bytes)
        .map_err(InternalAppError::new)?;

    let message = body_params.get("message").cloned();
    let thread_id = query.get("id").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (message, thread_id, csrf_token) = match (message, thread_id, csrf_token) {
        (Some(m), Some(ti), Some(ct)) => (m, ti, ct),
        _ => {
            dbg!(
                "Did not receive needed get and post params",
                query,
                body_params
            );
            return Ok(bad_request_400());
        }
    };

    if !abac::can_add_forum_thread_message(session.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        add_session_notification(
            db_pool,
            session::SessionNotification {
                session_id: session.id.clone(),
                message: "Insufficient permissions to send forum thread message".to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;

        return Ok(redirect_to(current_url.clone().to_string()));
    }

    match forum::find_thread(db_pool.clone(), thread_id.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        Some(_) => {}
        None => {
            add_session_notification(
                db_pool,
                session::SessionNotification {
                    session_id: session.id.clone(),
                    message: "Could not find thread with specified id".to_string(),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to("/forum".to_string()));
        }
    }

    let is_csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !is_csrf_token_valid {
        return Ok(bad_request_400());
    }

    forum::add_message(db_pool.clone(), message, session.user_id, thread_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let total_pages =
        forum::get_count_thread_pages(db_pool.clone(), thread_id.clone(), MESSAGES_PER_PAGE)
            .await
            .map_err(InternalAppError::new)?;
    let mut redir_url = current_url.clone();
    redir_url
        .query
        .insert("thread_page".to_string(), total_pages.to_string());
    let redir_url = redir_url.to_string();

    Ok(redirect_to(redir_url))
}
