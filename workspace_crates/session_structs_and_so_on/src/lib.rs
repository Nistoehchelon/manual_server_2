use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

pub const NAME_OF_COOKIE_CONTAINING_SESSION_ID: &str = "session_id";

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub logged_in: bool,
    pub previous_page_load_time_ms: i64,
}

impl Session {
    pub fn default_with_id(id: String) -> Self {
        Self {
        id: id,
        user_id: String::new(),
        logged_in: false,
        previous_page_load_time_ms: 0,
    }
    }
}

#[derive(Clone, Debug)]
pub enum SessionNotificationType {
    Information,
    Error,
    Success,
}
impl Display for SessionNotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SessionNotificationType::Information => "information",
            SessionNotificationType::Error => "error",
            SessionNotificationType::Success => "success",
        };
        write!(f, "{}", s)
    }
}
impl FromStr for SessionNotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Self::Error),
            "information" => Ok(Self::Information),
            "success" => Ok(Self::Success),
            val => Err(format!("Invalid type {val}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionNotification {
    pub session_id: String,
    pub message: String,
    pub notification_type: SessionNotificationType,
}
