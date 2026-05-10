use askama::Template;
use pagination::PaginationElement;

pub struct User {
    pub username: String,
}


pub struct Forum {
    pub id: String,
    pub name: String,
    pub parent_id: String,
}
impl From<forum::Forum> for Forum {
    fn from(f: forum::Forum) -> Self {
        Self {
            id: f.id,
            name: f.name,
            parent_id: f.parent_id,
        }
    }
}

#[derive(Template)]
#[template(path = "view_custom_page.html")]
pub struct ViewCustomPageTemplate {
    pub main_info: BaseTemplateData,
    pub page: CustomPage,
}

pub struct CustomPage {
    pub name: String,
    pub content: String,
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
    pub the_viewing_user_can_edit_it: bool,
}

pub struct Paginator {
    pub name: String,
    pub elements: Vec<PaginationElement>,
}

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate {
    pub main_info: BaseTemplateData,
    pub path: String,
    pub method: String,
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
    pub cloudflare_turnstile_enabled: bool,
    pub cloudflare_turnstile_site_key: String,
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
    pub username_length_min: usize,
    pub username_length_max: usize,
    pub password_length_min: usize,
    pub password_length_max: usize,
    pub main_info: BaseTemplateData,
    pub csrf_token: String,
}


#[derive(Template)]
#[template(path = "email_otp.html")]
pub struct EmailOTPTemplate {
    pub main_info: BaseTemplateData,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub username_length_min: usize,
    pub username_length_max: usize,
    pub password_length_min: usize,
    pub password_length_max: usize,
    pub csrf_token: String,
    pub main_info: BaseTemplateData,
}
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub main_info: BaseTemplateData,
    pub username: String,
    pub user_id: String,
    pub user_can_manage_site_pages: bool,
    pub user_can_view_app_executable_version: bool,
    pub app_executable_version: Option<String>,
}

#[derive(Template)]
#[template(path = "manage_site_pages.html")]
pub struct ManageSitePagesTemplate {
    pub main_info: BaseTemplateData,
    pub site_pages: Vec<ForumThreadIdAndName>,
    pub custom_site_pages_forum_id: String,
}

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub main_info: BaseTemplateData,
    pub parent_forum: Option<Forum>,
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
    pub thread_name_length_max: usize,
    pub thread_message_length_max: usize,
    pub main_info: BaseTemplateData,
    pub forum: Forum,
    pub draft_thread_name: String,
    pub draft_thread_content: String,
    pub draft_thread_custom_id: String,
    pub can_set_custom_id: bool,
    pub csrf_token: String,
} 

#[derive(Template)]
#[template(path = "edit_forum_thread_message.html")]
pub struct EditForumThreadMessageTemplate {
    pub thread_message_length_max: usize, 
    pub main_info: BaseTemplateData,
    pub thread: ForumThreadIdAndName,
    pub thread_page: String,
    pub message_content: String,
    pub message_id: String,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "create_forum.html")]
pub struct CreateForumTemplate {
    pub main_info: BaseTemplateData,
    pub csrf_token: String,
    pub parent_forum_id: String,
}
