use crate::user_structs_and_so_on::User;



pub fn get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Option<User> {
    // the point of this separation is to allow to at some point use a DB like Postgres if I wanted to
    storage_specific_get_user_by_user_id(user_id, db_pool)
}

pub fn try_to_create_new_user(
    username: String,
    password: String,
    access_level: usize,
     db_pool: &db_manager::DbPool,
) -> Result<User, String> {
    if username.clone().trim().is_empty() || password.clone().trim().is_empty() {
        return Err("Please enter username and password!!".to_string());
    }

    if !is_username_available(username.clone(), db_pool) {
        return Err(format!("Username {username} is taken already!"));
    }

    let user = storage_specific_create_new_user(
        username,
        password_hashing::hash_password(password).hash,
        access_level,
        db_pool,
    );

    Ok(user)
}

pub fn is_username_available(username: String,  db_pool: &db_manager::DbPool,) -> bool {
    let username_available = storage_specific_is_username_available(username,  db_pool);
    username_available
}

pub fn get_by_username(username: String, db_pool: &db_manager::DbPool,) -> Result<User, String> {
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

pub fn get_by_username_and_password(username: String, password: String, db_pool: &db_manager::DbPool,) -> Result<User, String> {
    let user = match get_by_username(username.clone(), db_pool) {
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
    id_generator::generate_id()
}

// TXT storage specific functions. Theoretically, this separation should allow for mysql/Postgres use in the future if I wanted to

fn storage_specific_is_username_available(username: String,  db_pool: &db_manager::DbPool,) -> bool {
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
    db_pool: &db_manager::DbPool,
) -> User {
    let user = User {
        id: generate_user_id(),
        username: username.clone(),
        password_hash: password.clone(),
        bio: String::from("UFO is watching"),
        access_level,
    };
    storage_specific_add_user(user.clone(), db_pool, );

    user
}

fn storage_specific_add_user(user: User, db_pool: &db_manager::DbPool,) {
    let mut users = read_users();
    users.push(user.clone());
    write_users(users.clone());
}

fn storage_specific_get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Option<User> {
    let all_users = read_users();

    for u in all_users {
        if u.id == user_id {
            return Some(u.clone());
        }
    }

    None
}

fn read_users() -> Vec<User> {
    todo!();
}

fn write_users(users: Vec<User>) {
    
}
