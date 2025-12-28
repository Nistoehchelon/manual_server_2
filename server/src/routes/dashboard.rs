use crate::{
    redirect::to,
};
use html_templating::html_template_structs::{self, MainInfo};
use session::Session;
use user;

pub fn get(session: Session) -> String {
    if !session.logged_in {
        return to("/".to_string());
    }
    let user = user::get_user_by_user_id(session.user_id.clone()).unwrap();
    let notifications = session::read_and_delete_session_notifications(session.id.clone());
    let html = html_templating::render(
        html_template_structs::DashboardTemplate{main_info: MainInfo {title: format!("Your Dashboard"), notifications}, username: user.username.clone()}
    );
    crate::http_response::ok_html(html)
}
