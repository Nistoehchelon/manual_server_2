use count_string_length::count_string_length::{count_string_length, valid_string_length};
use error_structs::PublicError;
use sqlx::types::time::OffsetDateTime;

// TODO: I wanna instead... I think it would make more sense to allow for dynamic setting of this using both .env and in-site db-held variable settings without having to restart the executable. But for simplicity for now let's keep it like this, hard-coded.
pub const MESSAGES_PER_PAGE: u64 = 15;

#[derive(Clone, Debug, Default)]
pub struct ForumThreadMessage {
    pub primary_key: i64,
    pub id: String,
    pub thread_id: String,
}

#[derive(Clone, Debug)]
pub struct ForumThreadMessageVersion {
    pub primary_key: i64,
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: OffsetDateTime,
}
#[derive(Clone, Debug, PartialEq, Eq)]

pub struct ValidForumThreadMessageContent {
    inner: String,
}

impl ValidForumThreadMessageContent {
    pub fn parse(s: String, min_length_forum_message: usize, max_length_forum_message: usize) -> Result<Self, PublicError> {
        let trimmed = s.trim().to_string();

        if !valid_string_length(
            trimmed.clone(),
            min_length_forum_message,
            max_length_forum_message,
        ) {
            return Err(PublicError::new(format!(
                "Couldn't validate message length. Allowed forum message length is between {} to {} characters but {} given.",
                min_length_forum_message,
                max_length_forum_message,
                count_string_length(trimmed)
            )));
        }
        Ok(Self {
            inner: s, // I do not actually wanna trim, just make sure that I do not count beginning and ending whitespace, but I think I am fine with visually showing it. TODO: strip it and don't consider it as valid?
        })
    }
}
impl std::fmt::Display for ValidForumThreadMessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Debug)]
pub struct ValidForumThreadName {
    inner: String,
}
pub const MIN_LENGTH_FORUM_THREAD_NAME: usize = 2;
pub const MAX_LENGTH_FORUM_THREAD_NAME: usize = 150;
impl ValidForumThreadName {
    pub fn parse(s: String) -> Result<Self, PublicError> {
        let s = s.trim().to_string();
        if !valid_string_length(
            s.clone(),
            MIN_LENGTH_FORUM_THREAD_NAME,
            MAX_LENGTH_FORUM_THREAD_NAME,
        ) {
            return Err(PublicError::new(format!(
                "Couldn't validate thread name length. Allowed length is between {} to {} but {} given.",
                MIN_LENGTH_FORUM_THREAD_NAME,
                MAX_LENGTH_FORUM_THREAD_NAME,
                count_string_length(s.clone())
            )));
        }
        if s.chars().any(|c| c.is_control() || c == '\n') {
            return Err(PublicError::new(
                "Thread name must not contain newlines/control characters.",
            ));
        }
        Ok(Self { inner: s })
    }
}
impl std::fmt::Display for ValidForumThreadName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ForumThread {
    pub id: String,
    pub name: String,
    pub forum_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct Forum {
    pub id: String,
    pub name: String,
    pub parent_id: String,
}
