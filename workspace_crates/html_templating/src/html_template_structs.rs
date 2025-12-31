use askama::Template;

use forum::{Forum, ForumThread};
use session::SessionNotification;
use user::User;

pub struct MainInfo {
    pub title: String,
    pub notifications: Vec<SessionNotification>,
}
#[derive(Template)]
#[template(path = "index_logged_in.html")]

pub struct IndexLoggedInTemplate {
    pub main_info: MainInfo,
    pub user: User,
}
#[derive(Template)]
#[template(path = "index_not_logged_in.html")]

pub struct IndexNotLoggedInTemplate {
    pub main_info: MainInfo,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate {
    pub main_info: MainInfo,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub main_info: MainInfo,
}
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub main_info: MainInfo,
    pub username: String,
}

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub main_info: MainInfo,
    pub user_can_create_child_forum: bool,
    pub user_can_create_thread: bool,
    pub forum: Forum,
    pub forum_threads: Vec<ForumThread>,
}

#[derive(Template)]
#[template(path = "create_thread.html")]
pub struct CreateThreadTemplate {
    pub main_info: MainInfo,
    pub forum: Forum,
}
