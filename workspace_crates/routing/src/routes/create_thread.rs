
use forum::{Forum, get_forum_by_id};
use html_templating::html_template_structs;
use http_stuff::{
    http_request_parsing::{self, Query},
    http_response::{HttpResponse, ResponseData, internal_server_error_500, ok_html, redirect_to, response},
};
use session::{Session, SessionNotification, add_session_notification};
use user;

pub async  fn get(session: Session, query: Query, db_pool: &sqlx::PgPool) -> HttpResponse {
    let _user = user::get_user_by_user_id(session.user_id.clone(), db_pool).await.unwrap();
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = match get_forum_by_id(db_pool, forum_id.clone()).await {
        Ok(opt) => opt,
        Err(_) => return internal_server_error_500(),
    };

    if !forum.is_some() {
       add_session_notification(
            db_pool,
            SessionNotification {
                session_id: session.id.clone(),
                message: format!("Forum not found"),
                notification_type: session::SessionNotificationType::Error,
            },
        ).await;
        return redirect_to("/forum".to_string());
    }
    let forum = forum.unwrap();
    let notifs = match session::read_and_delete_session_notifications(db_pool, session.id.clone()).await {
        Ok(n) => n,
        Err(_) => return internal_server_error_500(),
    };

    ok_html(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.clone(),
        notifs,
    ))
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


pub fn post(_session: Session, body_bytes: Vec<u8> , _db_pool: &sqlx::PgPool) -> HttpResponse {
    let params = match http_request_parsing::parse_body_into_string_post_params(body_bytes) {
        Ok(params) => params,
        Err(e) => {
            dbg!(e);
            return internal_server_error_500();
        }
    };
    dbg!(params.clone());
    let mut resp_body = format!(r##"
        
    "##);
    for param in params.iter() {
        let k = param.0;
        let v = param.1;
        resp_body = format!("{resp_body} ||| {k} .... {v}");
    }

    
    return response(ResponseData { code: http_stuff::http_response::ResponseCode::Ok200, content_type: "text/html".to_string(), body: Some(format!("{resp_body}").into_bytes()), set_cookie_headers: None });
}
