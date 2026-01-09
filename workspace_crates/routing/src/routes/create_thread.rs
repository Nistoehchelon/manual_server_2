
use error_structs::InternalAppError;
use forum::{Forum, get_forum_by_id};
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self, Query},
    http_response::{Charset, ResponseData, ok_html, redirect_to},
};
use session::{Session, SessionNotification, add_session_notification};
use user;

pub async  fn get(session: Session, query: Query, db_pool: &sqlx::PgPool) -> Result<ResponseData, error_structs::InternalAppError> {
    if !session.logged_in {
        return Ok(redirect_to("/".to_string()))
    }
    let _user = user::get_user_by_user_id(db_pool, session.user_id.clone()).await.map_err(InternalAppError::new)?.map_err(InternalAppError::new)?;
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = get_forum_by_id(db_pool, forum_id.clone()).await.map_err(InternalAppError::new)?;

    if !forum.is_some() {
       add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Forum not found"),
                notification_type: session::SessionNotificationType::Error,
            },
        ).await.map_err(InternalAppError::new)?;
        return Ok(redirect_to("/forum".to_string()));
    }
    let forum = forum.unwrap();
    let notifs = session::read_and_delete_session_notifications(db_pool, session.id.clone()).await.map_err(InternalAppError::new)?;

    Ok(ok_html(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.clone(),
        notifs,
    )))
}

fn get_html(
    title: String,
    forum: Forum,
    notifications: Vec<session::SessionNotification>,
) -> String {
    let html = html_templating::render(html_template_structs::CreateThreadTemplate {
        main_info: html_template_structs::MainInfo {
            title,
            notifications: into_vec::into_vec(notifications),
        },
        forum: forum.into(),
    });

    return html;
}


pub async fn post(_session: Session, body_bytes: Vec<u8> , _db_pool: &sqlx::PgPool) -> Result<ResponseData, error_structs::InternalAppError> {
    let params = http_request_parsing::parse_body_into_string_form_input_params(body_bytes).map_err(InternalAppError::new)?;
    dbg!(params.clone());
    let mut resp_body = format!(r##"
        
    "##); 
    for param in params.iter() {
        let k = param.0;
        let v = param.1;
        resp_body = format!("{resp_body} ||| {k} .... {v}");
    }

    
    return Ok(ResponseData { 
        status_code: http_stuff::http_response::StatusCode::Ok200, 
        content_type: http_stuff::http_response::ContentType::TextHtml, 
        body: Some(format!("{resp_body}").into_bytes()),
        charset: Some(Charset::Utf8), 
        set_cookie_headers: None,
        location: None, 
    });
}
