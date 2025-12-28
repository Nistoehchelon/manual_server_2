use crate::{
    common_type_aliases,
    field_count_trait::FieldCount,
    id_generator::generate_id,
    password_hashing,
    storage::{self, TextStorable},
};

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
// update this on altering struct User
impl FieldCount for User {
    fn field_count() -> usize {
        5
    }
}

impl TextStorable for User {
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

pub fn get_user_by_user_id(user_id: String) -> Option<User> {
    // the point of this separation is to allow to at some point use a DB like Postgres if I wanted to
    storage_specific_get_user_by_user_id(user_id)
}

pub fn try_to_create_new_user(
    username: String,
    password: String,
    access_level: usize,
) -> Result<User, String> {
    if username.clone().trim().is_empty() || password.clone().trim().is_empty() {
        return Err("Please enter username and password!!".to_string());
    }

    if !is_username_available(username.clone()) {
        return Err(format!("Username {username} is taken already!"));
    }

    let user = storage_specific_create_new_user(
        username,
        password_hashing::hash_password(password).hash,
        access_level,
    );

    Ok(user)
}

pub fn is_username_available(username: String) -> bool {
    let username_available = storage_specific_is_username_available(username);
    username_available
}

pub fn get_by_username(username: String) -> Result<User, String> {
    let users = read_users();
    let mut user = User::default();
    let mut found = false;
    for u in users {
        if username == u.username {
            found = true;
            user = u.clone();
        }
    }
    if found {
        Ok(user)
    } else {
        Err(String::from("User not found"))
    }
}

pub fn get_by_username_and_password(username: String, password: String) -> Result<User, String> {
    let user = match get_by_username(username.clone()) {
        Ok(user) => user,
        Err(_) => {
            return Err("Could not find user with specified username and password".to_string());
        }
    };
    if password_hashing::verify_password(password, user.password_hash.clone(), "".to_string())
        != true
    {
        return Err(
            "User found but hash of input password and existing password do not match".to_string(),
        );
    }
    Ok(user)
}

fn generate_user_id() -> String {
    generate_id()
}

// TXT storage specific functions. Theoretically, this separation should allow for mysql/Postgres use in the future if I wanted to

fn storage_specific_is_username_available(username: String) -> bool {
    let users = read_users();
    let mut username_available = true;
    for u in users.clone() {
        if username.clone() == u.username {
            username_available = false;
            break;
        }
    }
    username_available
}

fn storage_specific_create_new_user(
    username: String,
    password: String,
    access_level: usize,
) -> User {
    let user = User {
        id: generate_user_id(),
        username: username.clone(),
        password_hash: password.clone(),
        bio: String::from("UFO is watching"),
        access_level,
    };
    storage_specific_add_user(user.clone());

    user
}

fn storage_specific_add_user(user: User) {
    let mut users = read_users();
    users.push(user.clone());
    write_users(users.clone());
}

fn storage_specific_get_user_by_user_id(user_id: String) -> Option<User> {
    let all_users = read_users();

    for u in all_users {
        if u.id == user_id {
            return Some(u.clone());
        }
    }

    None
}

fn read_users() -> Vec<User> {
    storage::read_contents::<User>()
}

fn write_users(users: Vec<User>) {
    storage::write_contents::<User>(users).unwrap();
}
