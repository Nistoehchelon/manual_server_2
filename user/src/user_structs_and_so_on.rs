
use storage::storage::TextStorable;

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

impl TextStorable for User {
    fn field_count() -> usize {
        5
    }
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.username.clone(),
            self.password_hash.clone(),
            self.bio.clone(),
            self.access_level.clone().to_string(),
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
        let access_level_str = fields[4].clone();
        let access_level = access_level_str
            .parse::<usize>()
            .expect(format!("Could not parse String {access_level_str} into usize").as_str());
        Some(Self {
            id: fields[0].clone(),
            username: fields[1].clone(),
            password_hash: fields[2].clone(),
            bio: fields[3].clone(),
            access_level: access_level,
        })
    }
    fn file_name() -> &'static str {
        "users"
    }
}
