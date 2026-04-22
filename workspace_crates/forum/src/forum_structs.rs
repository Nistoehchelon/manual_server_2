
use count_string_length::count_string_length::count_string_length;
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
#[derive(Clone, Debug)]
pub struct ValidForumThreadMessageContent {
    inner: String,
}
pub const MAX_LENGTH_FORUM_MESSAGE: u64 = 10000; 
impl ValidForumThreadMessageContent {
    pub fn parse(s: String) -> Result<Self, PublicError> {
        if count_string_length(s.clone()) > MAX_LENGTH_FORUM_MESSAGE.try_into().map_err(PublicError::new)? {
            return Err(PublicError::new(format!("Max allowed forum message length is {} but {} given.", MAX_LENGTH_FORUM_MESSAGE, count_string_length(s.clone()))));
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
pub const MAX_LENGTH_FORUM_THREAD_NAME: u64 = 150; 
impl ValidForumThreadName {
    pub fn parse(s: String) -> Result<Self, PublicError> {
        if count_string_length(s.clone()) > MAX_LENGTH_FORUM_THREAD_NAME.try_into().map_err(PublicError::new)? {
            return Err(PublicError::new(format!("Max allowed forum thread name length is {} but {} given.", MAX_LENGTH_FORUM_THREAD_NAME, count_string_length(s.clone()))));
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
