use crate::{
    redirect::to,
};
use session::Session;
use user;

pub fn get(session: Session) -> String {
    if !session.logged_in {
        return to("/".to_string());
    }
    let user = user::get_user_by_user_id(session.user_id.clone()).unwrap();

    // Show dashboard (only reached if logged in)
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
             <h1>Hello, {}!</h1>
             <p>Welcome to your dashboard.</p>
             <p><a href='/logout'>Logout</a></p>
             ",
            user.username,
    )
}
