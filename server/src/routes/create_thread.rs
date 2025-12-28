use crate::common_html_css::surround_content_with_header_and_footer;
use crate::forum::get_forum_by_id;
use crate::http_request_parsing::Query;
use crate::http_response::ok;
use crate::redirect::to;
use crate::session::{Session, add_session_notification};
use crate::user;

pub fn get(session: Session, query: Query) -> String {
    let user = user::get_user_by_user_id(session.user_id.clone()).unwrap();
    let forum_id = query
        .get("forum_id")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let forum = get_forum_by_id(forum_id.clone());
    if !forum.is_some() {
        add_session_notification(session.id.clone(), format!("Forum not found"));
        return to("/forum".to_string());
    }
    let forum = forum.unwrap();
    if forum.access_level > user.access_level {
        add_session_notification(session.id.clone(), format!("Forum not found"));
        return to("/forum".to_string());
    }

    if !user.can(user::Action::CreateThread) {
        return to("/forum".to_string());
    }
    ok(get_html(
        format!(r##"Create a thread in forum {}"##, forum.name.clone()),
        forum.id.clone(),
    ))
}

fn get_html(title: String, forum_id: String) -> String {
    let html = format!(
        r##"
    <a href="/forum?id={forum_id}">Back to forum</a><br>
    <form method="POST" action="/create_thread">
        Thread name:<br> <input name="name"><br>
        Thread  content:<br> <textarea name="content"></textarea><br>
        <button type="submit">Submit</button><br>
        <input type = "hidden" name ="forum_id" value = "{forum_id}">
    </form>
    "##
    );
    surround_content_with_header_and_footer(format!("{}", html), title)
}
