use crate::{
    http_request_parsing::parse_username_and_password_from_body,
    redirect::{self, to},

};
 use html_templating::html_template_structs;
use     session::{
        Session, SessionNotification, add_session_notification, edit_session, read_and_delete_session_notifications
    };
    use    user::get_by_username_and_password;

pub fn get(session: Session) -> String {
    if session.logged_in {
        return redirect::to("/".to_string());
    }
    crate::http_response::ok_html(
        get_html(read_and_delete_session_notifications(
            session.id.clone()
        )))
}

pub fn post(session: Session, body: String, db_pool: &db_manager::DbPool) -> String {
    if session.logged_in {
        add_session_notification(
            session.id.clone(),
            String::from("You are already logged in."),
            db_pool,
        );
        return to(String::from("/"));
    }
    let parse_result = parse_username_and_password_from_body(body);
    if parse_result.is_err() {
        add_session_notification(session.id.clone(), parse_result.unwrap_err(), db_pool,);
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
    let user_result = get_by_username_and_password(username, password, db_pool);
    if user_result.is_err() {
        add_session_notification(
            session.id.clone(),
            "Username and/or password incorrect".to_string(),
            db_pool,
            
        );
        return to("/login".to_string());
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_id = user.id.clone();
    session.logged_in = true;
    edit_session(session.clone());

    add_session_notification(session.id, "Successfully logged in! Welcome!".to_string(), db_pool);
    redirect::to("/".to_string())
}

fn get_html(notifications: Vec<SessionNotification>) -> String {
    let html = html_templating::render(html_template_structs::LoginTemplate { main_info: html_template_structs::MainInfo { title: format!("Login"), notifications }  });
    return html;
}
