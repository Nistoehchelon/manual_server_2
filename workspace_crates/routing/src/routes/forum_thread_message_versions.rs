use error_structs::InternalAppError;
use html_templating::{html_template_structs::{self}, render};
use http_stuff::http_response::{ResponseData, ok_html, redirect_to};
use into_vec::into_vec;
use session::{SessionNotification, SessionNotificationType, add_session_notification, read_and_delete_session_notifications};
use url::Url;
use pagination::Pagination;
use user::get_user_by_user_id;

use crate::RouterData;

pub async fn get(RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,) -> Result<ResponseData, InternalAppError> {
    let query = request_data.query.clone();
    let message_id = match query.get("message_id").cloned() {
        Some(tid) => tid,
        None => {
                add_session_notification(
                db_pool.clone(),
                SessionNotification {
                    session_id: session.id.clone(),
                    message: format!("Could not find thread_id in query string"),
                    notification_type: SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
        return Ok(redirect_to("/forum".to_string()));
        },
    };
    let page = match query.get("page").cloned() {
        None => 1,
        Some(p) => {
            let mut p_int = match p.parse::<u64>() {
                Ok(p_int) => p_int,
                Err(_) => {
                    return Ok(redirect_to("/forum".to_string()));
                }
            };
            if p_int == 0 {
                p_int = 1;
            }
            p_int
        }
    };
    let thread_page = match query.get("thread_page").cloned() {
        None => 1,
        Some(p) => {
            let mut p_int = match p.parse::<u64>() {
                Ok(p_int) => p_int,
                Err(_) => {
                    return Ok(redirect_to("/forum".to_string()));
                }
            };
            if p_int == 0 {
                p_int = 1;
            }
            p_int
        }
    };

    let msg = match forum::find_thread_message(db_pool.clone(), message_id.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        Some(m) => m,
        None => {
            add_session_notification(
                db_pool.clone(),
                session::SessionNotification {
                    session_id: session.id.clone(),
                    message: format!("Could not find message with specified id"),
                    notification_type: session::SessionNotificationType::Error,
                },
            )
            .await
            .map_err(InternalAppError::new)?;
            return Ok(redirect_to(format!("/forum")));
        }
    };

    let thread = forum::get_thread(db_pool.clone(), msg.thread_id.clone()).await.map_err(InternalAppError::new)?;

    const VERSIONS_PER_PAGE: u64 = 15;

    let message_versions = forum::get_thread_message_versions(db_pool.clone(), message_id.clone(), page.clone(), VERSIONS_PER_PAGE).await.map_err(InternalAppError::new)?;

        let versions_count =
        forum::get_count_thread_message_versions(db_pool.clone(), message_id.clone())
            .await
            .map_err(InternalAppError::new)?;

    let current_url = Url::new(request_data.path.clone(), request_data.query.clone());
    let pagi = Pagination {
        records_count: versions_count.clone(),
        records_per_page: VERSIONS_PER_PAGE,
        current_page: page.clone(),
        current_url: current_url.clone(),
        page_param: "page".to_string(),
    };
    let versions_pagination_elements = pagi.get_pagination_elements();

    let mut message_versions_to_render: Vec<
        html_template_structs::ForumThreadMessageVersion
    > = Vec::with_capacity(usize::try_from(VERSIONS_PER_PAGE).map_err(InternalAppError::new)?);

    for msg in message_versions {
        let user = get_user_by_user_id(db_pool.clone(), msg.user_id).await.map_err(InternalAppError::new)?;
        message_versions_to_render.push(html_template_structs::ForumThreadMessageVersion {
            user_id: user.id.clone(),
            user_username: user.username,
            content: msg.content,
            created_at: msg.created_at.to_string(),
        });
    }

    return Ok(ok_html(render(html_template_structs::ForumThreadMessageVersionsTemplate {
        main_info: html_template_structs::MainInfo {
            title: "Message versions".to_string(),
            notifications: into_vec(read_and_delete_session_notifications(db_pool.clone(), session.id.clone()).await.map_err(InternalAppError::new)?),
        },
        message_id,
        thread_id: thread.id,
        thread_page: thread_page.to_string(),
        message_versions: message_versions_to_render,
        versions_paginator: html_template_structs::Paginator {
            name: "Message versions paginator".to_string(),
            elements: versions_pagination_elements,
        },
    })));
}
