
use 
    html_templating::html_template_structs::{IndexLoggedInTemplate, IndexNotLoggedInTemplate, MainInfo};

    use session::{Session, SessionNotification, read_and_delete_session_notifications};
    use user::{ get_user_by_user_id};
 
pub fn get(session: Session) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
        {}",
        get_html(
            session.clone(),
            read_and_delete_session_notifications(session.id.clone())
        ),
    )
}

fn get_html(session: Session, notifications: Vec<SessionNotification>) -> String {
    if session.logged_in {
        let user = get_user_by_user_id(session.user_id.clone()).unwrap();
        let index_template = IndexLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome back!"),
                notifications,
            },
            user,
        };
        html_templating::render(index_template)
    } else {
        let index_template = IndexNotLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome!"),
                notifications,
            },
        };
        html_templating::render(index_template)
    }
}
