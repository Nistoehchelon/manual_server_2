use askama::Template;

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

pub struct ForumThread {
    pub id: String,
    pub name: String,
}
impl From<forum::ForumThread> for ForumThread {
    fn from(src: forum::ForumThread) -> Self {
        Self {
            id: src.id,
            name: src.name,
        }
    }
}
pub struct MainInfo {
    pub title: String,
    pub notifications: Vec<SessionNotification>,
}

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
    pub user_can_create_forum: bool,
    pub user_can_create_thread: bool,
    pub forum: Forum,
    pub forum_threads: Vec<ForumThread>,
    pub subforums: Vec<Forum>,
}

#[derive(Template)]
#[template(path = "create_thread.html")]
pub struct CreateThreadTemplate {
    pub main_info: MainInfo,
    pub forum: Forum,
}
