
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
    pub content: ValidForumThreadMessageContent,
    pub created_at: OffsetDateTime,
}
#[derive(Clone, Debug, PartialEq, Eq)]

pub struct ValidForumThreadMessageContent {
    inner: String,
}
pub const MIN_LENGTH_FORUM_MESSAGE: usize = 0; // Might change later, depends 
pub const MAX_LENGTH_FORUM_MESSAGE: usize = 10000; 
impl ValidForumThreadMessageContent {
    pub fn parse(s: String) -> Result<Self, PublicError> {
        if !valid_string_length(s.clone(), MIN_LENGTH_FORUM_MESSAGE, MAX_LENGTH_FORUM_MESSAGE){
            return Err(PublicError::new(format!("Couldn't validate message length. Allowed forum message length is between {} to {} characters but {} given.", MIN_LENGTH_FORUM_MESSAGE, MAX_LENGTH_FORUM_MESSAGE, count_string_length(s.clone()))));
        }
        Ok(Self {
            inner: s,
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
        if !valid_string_length(s.clone(), MIN_LENGTH_FORUM_THREAD_NAME, MAX_LENGTH_FORUM_THREAD_NAME) {
            return Err(PublicError::new(format!("Coudln't validate thread name length. Allowed length is between {} to {} but {} given.", MIN_LENGTH_FORUM_THREAD_NAME, MAX_LENGTH_FORUM_THREAD_NAME, count_string_length(s.clone()))));
        }
        Ok(Self {
            inner: s,
        })
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
