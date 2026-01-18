use sqlx::types::time::OffsetDateTime;

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
