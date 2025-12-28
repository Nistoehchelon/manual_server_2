
use storage::TextStorable;
use id_generator::generate_id;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub logged_in: bool,
}

impl TextStorable for Session {
    fn field_count() -> usize {
        3
    }
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.user_id.clone(),
            self.logged_in.clone().to_string(),
        ]
    }
    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() != Self::field_count() {
            panic!(
                "Mismatch of expected struct fields. Expected: {}, got: {}",
                Self::field_count(),
                fields.len()
            );
        }
        let logged_in_bool = fields[2]
            .clone()
            .parse()
            .expect("Could not parse into bool");
        Some(Self {
            id: fields[0].clone(),
            user_id: fields[1].clone(),
            logged_in: logged_in_bool,
        })
    }
    fn file_name() -> &'static str {
        "sessions"
    }
}

#[derive(Clone, Debug)]
pub struct SessionNotification {
    pub session_id: String,
    pub message: String,
}

impl TextStorable for SessionNotification {
    fn field_count() -> usize {
        2
    }
    fn to_fields(&self) -> Vec<String> {
        vec![self.session_id.clone(), self.message.clone()]
    }
    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() != Self::field_count() {
            panic!(
                "Mismatch of expected struct fields. Expected: {}, got: {}",
                Self::field_count(),
                fields.len()
            );
        }
        Some(Self {
            session_id: fields[0].clone(),
            message: fields[1].clone(),
        })
    }
    fn file_name() -> &'static str {
        "session_notifications"
    }
}

pub fn notifications_to_str(notifications: Vec<SessionNotification>) -> String {
    let mut notifications_str = String::from("<div id=\"notifications\">");
    for notification in notifications {
        notifications_str.push_str(notification.message.as_str());
        notifications_str.push_str("<br>");
    }
    notifications_str.push_str("</div>");
    notifications_str
}

pub fn edit_session(modified_session: Session) {
    storage_specific_edit_session(modified_session);
}

pub fn add_session_notification(session_id: String, message: String, db_pool: &db_manager::DbPool) {
    storage_specific_add_session_notification(session_id, message, db_pool);
}

pub fn read_and_delete_session_notifications(session_id: String) -> Vec<SessionNotification> {
    let notifications_associated_with_the_session_id =
        storage_specific_read_and_delete_session_notifications(session_id);
    notifications_associated_with_the_session_id
}

pub fn session_with_specified_id_exists(session_id: String, db_pool: &db_manager::DbPool) -> bool {
    if get_session_by_session_id(session_id.clone(), db_pool).is_none() {
        return false;
    }
    true
}

pub fn get_session_by_session_id(session_id: String, db_pool: &db_manager::DbPool) -> Option<Session> {
    storage_specific_get_session_by_session_id(session_id)
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub fn create_new_session_with_id(new_id: String, db_pool: &db_manager::DbPool) -> Session {
    let new_session = Session {
        id: new_id.clone(),
        user_id: String::new(),
        logged_in: false,
    };

    storage_specific_add_session(new_session.clone(), db_pool);
    new_session
}

// separation between general functions above and storage specific stuff (txt vs mysql)

fn storage_specific_add_session(session: Session, db_pool: &db_manager::DbPool) {
    let mut sessions = read_sessions();

    sessions.push(session.clone());

    write_sessions(sessions.clone());
}

fn storage_specific_get_session_by_session_id(session_id: String) -> Option<Session> {
    let sessions = read_sessions();
    sessions
        .iter()
        .find(|session| session.id == session_id)
        .cloned()
}

fn storage_specific_read_and_delete_session_notifications(
    session_id: String,
) -> Vec<SessionNotification> {
    let all_session_notifications = read_session_notifications();
    let mut notifications_associated_with_the_session_id: Vec<SessionNotification> = Vec::new();
    let mut new_all_session_notifications: Vec<SessionNotification> = Vec::new();
    for msg in all_session_notifications.clone() {
        if msg.session_id != session_id {
            new_all_session_notifications.push(msg);
            continue;
        }
        notifications_associated_with_the_session_id.push(msg.clone());
    }
    write_session_notifications(new_all_session_notifications);
    notifications_associated_with_the_session_id
}

fn storage_specific_edit_session(modified_session: Session) {
    let sessions = read_sessions();
    let mut new_sessions: Vec<Session> = Vec::new();
    for old_session in sessions.clone() {
        if modified_session.id == old_session.id {
            new_sessions.push(modified_session.clone());
            continue;
        }
        new_sessions.push(old_session);
    }
    write_sessions(new_sessions);
}

fn storage_specific_add_session_notification(session_id: String, message: String, db_pool: &db_manager::DbPool) {
    let notification = SessionNotification {
        session_id,
        message,
    };
    let mut all_notifications = read_session_notifications();
    all_notifications.push(notification);
    write_session_notifications(all_notifications);
}

fn write_session_notifications(session_notifications: Vec<SessionNotification>) {
    storage::write_contents::<SessionNotification>(session_notifications).unwrap();
}

fn read_session_notifications() -> Vec<SessionNotification> {
    storage::read_contents::<SessionNotification>()
}

fn write_sessions(sessions: Vec<Session>) {
    storage::write_contents::<Session>(sessions).unwrap();
}

fn read_sessions() -> Vec<Session> {
    storage::read_contents::<Session>()
}
