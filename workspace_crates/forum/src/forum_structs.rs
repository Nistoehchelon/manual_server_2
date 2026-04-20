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
