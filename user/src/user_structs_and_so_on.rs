
pub const ADMIN_LVL: common_type_aliases::AccessLevel = 3;
pub const USER_LVL: common_type_aliases::AccessLevel = 1;

#[derive(Debug, PartialEq)]
pub enum Action {
    CreateThread,
    CreateForum,
    DeleteThread,
}

#[derive(Clone, Debug, Default)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub bio: String,
    pub access_level: usize,
}
impl User {
    pub fn can(&self, action: Action) -> bool {
        match action {
            Action::CreateForum => self.access_level >= ADMIN_LVL,
            Action::CreateThread => self.access_level >= USER_LVL,
            _ => panic!("Invalid action {action:?}"),
        }
    }
}
