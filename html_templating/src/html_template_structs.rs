use askama::Template;

use crate::{session::SessionNotification, user::User};

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