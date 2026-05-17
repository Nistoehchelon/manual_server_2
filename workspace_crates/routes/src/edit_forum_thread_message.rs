use abac::CanEditForumThreadMessageInput;
use error_structs::InternalAppError;
use forum::{ValidForumThreadMessageContent, forum_structs, get_thread};
use html_templating::html_template_structs::EditForumThreadMessageTemplate;
use http_stuff::{
    http_request_parsing,
    http_response::{self, HttpResponseData, bad_request_400, redirect_to},
};
use url::Url;

use routing_structs::RouterData;

pub async fn get(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let db_pool = router_data.db_pool.clone();
    let session = router_data.session.clone();
    if !session.logged_in {
        println!("Non-logged in user tried to view edit message page");
        return Ok(redirect_to("/".to_string()));
    }

    let the_user = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let csrf_token = csrf_stuff::get_or_create_token(
        db_pool.clone(),
        session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;
    let msg_id = {
        let msg_id = router_data.http_request_data.query.get("id");
        if let Some(msg_id) = msg_id {
            msg_id.to_owned()
        } else {
            println!("Did not receive message_id GET param");
            return Ok(redirect_to("/".to_string()));
        }
    };
    let msg = forum::forum_thread_message::get_message(db_pool.clone(), msg_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let original_message_poster_id =
        forum::get_initial_message_version(db_pool.clone(), msg_id.clone())
            .await
            .map_err(InternalAppError::new)?
            .user_id;
    let current_message_version =
        forum::get_latest_message_version(db_pool.clone(), msg_id.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !abac::can_edit_forum_thread_message(CanEditForumThreadMessageInput {
        original_message_poster_id,
        user_id: the_user.id,
    })
    .await
    .map_err(InternalAppError::new)?
    {
        println!("User can't edit the message");
        return Ok(redirect_to("/".to_string()));
    }
    let thread = get_thread(db_pool.clone(), msg.thread_id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let thread_page = {
        router_data
            .http_request_data
            .query
            .get("thread_page")
            .to_owned()
            .unwrap_or(&"1".to_string())
            .to_string()
    };
    let template = EditForumThreadMessageTemplate {
        main_info: router_data.base_template_data,
        thread: thread.into(),
        thread_page,
        message_content: current_message_version.content.to_string(),
        message_id: msg_id.clone(),
        csrf_token,
        thread_message_length_max: router_data.site_config.max_length_forum_message,
    };
    Ok(http_response::ok_html(html_templating::render(template)))
}

pub async fn post(
    router_data: RouterData,
) -> Result<HttpResponseData, error_structs::InternalAppError> {
    let RouterData {
        db_pool, session, ..
    } = router_data.clone();

    let current_url = Url::new(
        router_data.http_request_data.path.clone(),
        router_data.http_request_data.query.clone(),
    );

    let body_params = http_request_parsing::parse_body_into_string_form_input_params(
        router_data.http_request_data.body_bytes.clone(),
    )
    .map_err(InternalAppError::new)?;

    let message = body_params.get("message").cloned();
    let message_id = body_params.get("message_id").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (message_id, csrf_token, message) = match (message_id, csrf_token, message) {
        (Some(msg_id), Some(ct), Some(m)) => (msg_id, ct, m),
        _ => {
            dbg!(
                "Did not receive needed get and post params",
                router_data.http_request_data.query.clone(),
                body_params
            );
            return Ok(bad_request_400());
        }
    };
    let message = match ValidForumThreadMessageContent::parse(
        message,
        router_data.site_config.min_length_forum_message,
        router_data.site_config.max_length_forum_message,
    ) {
        Ok(m) => m,
        Err(e) => {
            session::add_session_notification_error(db_pool.clone(), session.id.clone(), e.message)
                .await;
            return Ok(redirect_to(session.previous_get_page.clone()));
        }
    };
    let msg =
        forum::forum_thread_message::get_message(router_data.db_pool.clone(), message_id.clone())
            .await
            .map_err(InternalAppError::new)?;
    let curr_msg_content: forum::ForumThreadMessageVersion =
        forum::get_latest_message_version(db_pool.clone(), msg.id.clone())
            .await
            .map_err(InternalAppError::new)?;
    let op_msg = forum::get_initial_message_version(router_data.db_pool.clone(), msg.id.clone())
        .await
        .map_err(InternalAppError::new)?;
    let return_url = {
        format!(
            "/forum_thread?id={}&page={}",
            msg.thread_id,
            forum::forum_thread_message::get_message_page(
                db_pool.clone(),
                msg.id,
                forum_structs::MESSAGES_PER_PAGE
            )
            .await
            .map_err(InternalAppError::new)?
        )
    };
    if message.to_string() == curr_msg_content.content {
        session::add_session_notification_info(
            db_pool.clone(),
            session.id.clone(),
            "The text you submitted is identical to the current one. Thus no changes were made."
                .to_string(),
        )
        .await;
        return Ok(redirect_to(return_url));
    }
    let user = user::get_user_by_user_id(
        router_data.db_pool.clone(),
        router_data.session.user_id.clone(),
    )
    .await
    .map_err(InternalAppError::new)?;
    if !abac::can_edit_forum_thread_message(CanEditForumThreadMessageInput {
        user_id: user.id.clone(),
        original_message_poster_id: op_msg.user_id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?
    {
        session::add_session_notification(
            router_data.db_pool.clone(),
            session::SessionNotification {
                session_id: router_data.session.id.clone(),
                message: "Insufficient permissions to edit forum thread message".to_string(),
                notification_type: session::SessionNotificationType::Error,
            },
        )
        .await
        .map_err(InternalAppError::new)?;

        return Ok(redirect_to(current_url.clone().to_string()));
    }

    let is_csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !is_csrf_token_valid {
        return Ok(bad_request_400());
    }

    println!("Entering edit_forum_thread_message");
    forum::forum_thread_message::edit_forum_thread_message(
        db_pool.clone(),
        message_id,
        message.to_string(),
        user.id,
    )
    .await
    .map_err(InternalAppError::new)?;

    Ok(redirect_to(return_url))
}
