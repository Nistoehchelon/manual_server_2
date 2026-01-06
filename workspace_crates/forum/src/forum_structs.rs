#[derive(Clone, Debug, Default)]
pub struct ForumThreadMessage {
    pub primary_key: i64,
    pub public_id: String,
    pub user_id: String,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct ForumThread {
    pub public_id: String,
    pub name: String,
    pub forum_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct Forum {
    pub public_id: String,
    pub name: String,
    pub parent_id: String,
}
