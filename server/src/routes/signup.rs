use crate::{
    http_request_parsing::parse_username_and_password_from_body,
    redirect::to,
};
use session::{
        Session, add_session_notification, edit_session, notifications_to_str,
        read_and_delete_session_notifications,
    };
use user::try_to_create_new_user;

pub fn get(session: Session) -> String {
    if session.logged_in {
        add_session_notification(
            session.id.clone(),
            "You are already logged in, so signup is unaccessible. Log out first?".to_string(),
        );
        return to("/".to_string());
    }

    // Show username form
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
         {}",
        get_signup_html(notifications_to_str(read_and_delete_session_notifications(
            session.id.clone()
        ))),
    )
}

fn get_signup_html(notifications: String) -> String {
    format!("<h1>Sign up</h1>
            {notifications}
             <form method='POST' action='/signup'>
                 <input type='text' name='username' placeholder='Enter your username' required><br>
                 <input type='password' name='password' placeholder='Enter your password' required><br>
                 <input type='submit' value='Create a new account!'>
             </form>")
}

pub fn post(session: Session, body: String) -> String {
    if session.logged_in {
        return to("/".to_string());
    }

    let parse_result = parse_username_and_password_from_body(body.clone());
    if parse_result.is_err() {
        add_session_notification(session.id.clone(), String::from("Invalid body"));
        return to("/".to_string());
    }
    let (username, password) = parse_result.unwrap();
    let access_level = 1;

    match try_to_create_new_user(username, password, access_level) {
        Ok(user) => {
            let mut modified_session = session.clone();
            modified_session.logged_in = true;
            modified_session.user_id = user.id.clone();
            edit_session(modified_session);
        }
        Err(error_message) => {
            add_session_notification(session.id, error_message);
            return to("/signup".to_string());
        }
    }

    to("/".to_string())
}
