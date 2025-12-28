use crate::{
    common_html_css::{get_footer_html, get_header_html},
    http_request_parsing::parse_username_and_password_from_body,
    redirect::{self, to},
    session::{
        Session, add_session_notification, edit_session, notifications_to_str,
        read_and_delete_session_notifications,
    },
    user::get_by_username_and_password,
};

pub fn get(session: Session) -> String {
    if session.logged_in {
        return redirect::to("/".to_string());
    }
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
        {}{}{}",
        get_header_html("Login Page".to_string()),
        get_html(notifications_to_str(read_and_delete_session_notifications(
            session.id.clone()
        ))),
        get_footer_html()
    )
}

pub fn post(session: Session, body: String) -> String {
    if session.logged_in {
        add_session_notification(
            session.id.clone(),
            String::from("You are already logged in."),
        );
        return to(String::from("/"));
    }
    let parse_result = parse_username_and_password_from_body(body);
    if parse_result.is_err() {
        add_session_notification(session.id.clone(), parse_result.unwrap_err());
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
    let user_result = get_by_username_and_password(username, password);
    if user_result.is_err() {
        add_session_notification(
            session.id.clone(),
            "Username and/or password incorrect".to_string(),
        );
        return to("/login".to_string());
    }
    let user = user_result.unwrap();
    let mut session = Session { ..session };
    session.user_id = user.id.clone();
    session.logged_in = true;
    edit_session(session.clone());

    add_session_notification(session.id, "Successfully logged in! Welcome!".to_string());
    redirect::to("/".to_string())
}

fn get_html(notifications_str: String) -> String {
    format!(
        r##"
        Login page: 
        {notifications_str}
        <form action="/login" method="POST">
        Username:<br><input type="text" name="username"><br>
        Password:<br><input type="text" name="password"><br>
        <input type="submit" value="Login">
        </form>
    "##
    )
}
