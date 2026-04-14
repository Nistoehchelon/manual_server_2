use askama::Template;
use pagination::PaginationElement;

pub struct User {
    pub username: String,
}

pub struct Forum {
    pub id: String,
    pub name: String,
}
impl From<forum::Forum> for Forum {
    fn from(f: forum::Forum) -> Self {
        Self {
            id: f.id,
            name: f.name,
        }
    }
}

pub struct ForumThreadIdAndName {
    pub id: String,
    pub name: String,
}
impl From<forum::ForumThread> for ForumThreadIdAndName {
    fn from(src: forum::ForumThread) -> Self {
        Self {
            id: src.id,
            name: src.name,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForumThreadMessageCombinedData {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub user_id: String,
    pub user_username: String,
    pub modifications_count: i64,
}

pub struct Paginator {
    pub name: String,
    pub elements: Vec<PaginationElement>,
}

#[derive(Template)]
#[template(path = "forum_thread.html")]
pub struct ForumThread {
    pub main_info: BaseTemplateData,
    pub thread_messages: Vec<ForumThreadMessageCombinedData>,
    pub csrf_token: String,
    pub current_url: String,
    pub thread_name: String,
    pub thread_page: u64,
    pub forum_id: String,
    pub thread_paginator: Paginator,
    pub user_can_add_message: bool,
}

#[derive(Clone, Debug)]
pub struct BaseTemplateData {
    pub page: String,
    pub title: String,
    pub notifications: Vec<SessionNotification>,
    pub nonce: String,
}

#[derive(Debug, Clone)]
pub struct SessionNotification {
    pub message: String,
    pub notification_type: String,
}
impl From<session::SessionNotification> for SessionNotification {
    fn from(notif: session::SessionNotification) -> Self {
        Self {
            message: notif.message,
            notification_type: notif.notification_type.to_string(),
        }
    }
}

pub struct ForumThreadMessageVersion {
    pub user_id: String,
    pub user_username: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "forum_thread_message_versions.html")]
pub struct ForumThreadMessageVersionsTemplate {
    pub main_info: BaseTemplateData,
    pub message_id: String,
    pub thread_id: String,
    pub thread_page: String,
    pub message_versions: Vec<ForumThreadMessageVersion>,
    pub versions_paginator: Paginator,
}

#[derive(Template)]
#[template(path = "index_logged_in.html")]
pub struct IndexLoggedInTemplate {
    pub main_info: BaseTemplateData,
    pub user: User,
}
#[derive(Template)]
#[template(path = "index_not_logged_in.html")]
pub struct IndexNotLoggedInTemplate {
    pub main_info: BaseTemplateData,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate {
    pub main_info: BaseTemplateData,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub csrf_token: String,
    pub main_info: BaseTemplateData,
}
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub main_info: BaseTemplateData,
    pub username: String,
}

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub main_info: BaseTemplateData,
    pub user_can_create_forum: bool,
    pub user_can_create_thread: bool,
    pub forum: Forum,
    pub forum_threads: Vec<ForumThreadIdAndName>,
    pub subforums: Vec<Forum>,
    pub forum_threads_pagination: Vec<pagination::PaginationElement>,
}

#[derive(Template)]
#[template(path = "create_thread.html")]
pub struct CreateThreadTemplate {
    pub main_info: BaseTemplateData,
    pub forum: Forum,
    pub csrf_token: String,
}
