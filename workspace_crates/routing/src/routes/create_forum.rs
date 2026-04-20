use abac::{CanAccessThisForumInput, can_access_this_forum};
use error_structs::InternalAppError;
use html_templating::{html_template_structs::CreateForumTemplate, render};
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{HttpResponseData, bad_request_400, ok_html, redirect_to},
};
use session::add_session_notification_success;

use crate::RouterData;

pub async fn post(rt: RouterData) -> Result<HttpResponseData, InternalAppError> {
    if !abac::can_create_forum(abac::CanCreateForumInput {
        db_pool: rt.db_pool.clone(),
        session_id: rt.session.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?
    {
        println!("User tried to access GET create_forum but does not have sufficient permissions");
        return Ok(redirect_to("/".to_string()));
    }
    let body_params =
        match parse_body_into_string_form_input_params(rt.http_request_data.body_bytes.clone()) {
            Ok(p) => p,
            Err(e) => {
                println!("Could not parse body, {}", e);
                return Ok(redirect_to("/".to_string()));
            }
        };
    let csrf_token = match body_params.get("csrf_token") {
        Some(t) => t.to_owned(),
        None => {
            println!("POST: csrf_token not provided");
            return Ok(redirect_to("/".to_string()));
        }
    };
    let is_csrf_token_valid = csrf_stuff::validate_token(
        rt.db_pool.clone(),
        rt.session.id.clone(),
        csrf_token.clone(),
    )
    .await
    .map_err(InternalAppError::new)?;
    if !is_csrf_token_valid {
        return Ok(bad_request_400());
    }
    let parent_forum_id = {
        let opt = body_params.get("parent_forum_id");
        match opt {
            Some(id) => id.to_owned(),
            None => {
                println!("parent_forum_id POST not provided");
                return Ok(redirect_to("/forum".to_string()));
            }
        }
    };
    let name = {
        let opt = body_params.get("name");
        match opt {
            Some(n) => n.to_owned(),
            None => {
                println!("name  not provided");
                return Ok(redirect_to("/forum".to_string()));
            }
        }
    };
    let id_source = {
        let opt = body_params.get("custom_id");
        match opt {
            Some(custom_id) if custom_id == "[auto]" => forum::ForumIdSource::Auto,
            Some(custom_id) => {
                let unique_id =
                    forum::UniqueForumId::parse(rt.db_pool.clone(), custom_id.to_owned())
                        .await
                        .map_err(InternalAppError::new)?;
                forum::ForumIdSource::Custom(unique_id)
            }
            None => {
                println!("custom_id not provided");
                return Ok(redirect_to("/".to_string()));
            }
        }
    };
    // none forum is for example site pages, not shown in 0 default forum
    if parent_forum_id != "none" {
        let _parent_forum = forum::get_forum_by_id(rt.db_pool.clone(), parent_forum_id.clone())
            .await
            .map_err(InternalAppError::new)?;
    }
    if !can_access_this_forum(CanAccessThisForumInput{ db_pool:rt.db_pool.clone(), user_id: rt.session.user_id.clone(), forum_id: parent_forum_id.clone() }).await.map_err(InternalAppError::new)? {
        println!("User doesn't have permissions to access this forum");
        return Ok(redirect_to("/".to_string()));
    }
    let newly_created_forum_id =
        forum::create_forum(rt.db_pool.clone(), parent_forum_id, id_source, name.clone())
            .await
            .map_err(InternalAppError::new)?;
    add_session_notification_success(
        rt.db_pool.clone(),
        rt.session.id,
        "Forum created! Well done.".to_string(),
    )
    .await;
    Ok(redirect_to(format!("/forum?id={newly_created_forum_id}")))
}

pub async fn get(rt: RouterData) -> Result<HttpResponseData, InternalAppError> {
    if !abac::can_create_forum(abac::CanCreateForumInput {
        db_pool: rt.db_pool.clone(),
        session_id: rt.session.id.clone(),
    })
    .await
    .map_err(InternalAppError::new)?
    {
        println!("User tried to access GET create_forum but does not have sufficient permissions");
        return Ok(redirect_to("/".to_string()));
    }
    let parent_forum_id = {
        let opt = rt.http_request_data.query.get("parent_id");
        match opt {
            Some(id) => id.to_owned(),
            None => return Ok(redirect_to("/forum".to_string())),
        }
    };
    let csrf_token = csrf_stuff::get_or_create_token(
        rt.db_pool.clone(),
        rt.session.id.clone(),
        csrf_stuff::DesignatedPurpose::Normal,
    )
    .await
    .map_err(InternalAppError::new)?;
    let tpl = CreateForumTemplate {
        main_info: rt.base_template_data,
        parent_forum_id,
        csrf_token,
    };
    Ok(ok_html(render(tpl)))
}
