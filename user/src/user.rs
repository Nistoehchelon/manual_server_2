

use crate::user_structs_and_so_on::User;



pub fn get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Option<User> {
    // the point of this separation is to allow to at some point use a DB like Postgres if I wanted to
    storage_specific_get_user_by_user_id(user_id, db_pool)
}

pub fn try_to_create_new_user(
    db_pool: &db_manager::DbPool,
    username: String,
    password: String,
) -> Result<User, String> {
    if username.clone().trim().is_empty() || password.clone().trim().is_empty() {
        return Err("Please enter username and password!!".to_string());
    }

    if !is_username_available(username.clone(), db_pool) {
        return Err(format!("Username {username} is taken already!"));
    }

    let hash_and_salt = password_hashing::hash_password(password);
    let default_user_role = 1;
    let new_user = User {
        id: generate_user_id(),
        username,
        password_hash: hash_and_salt.hash,
        password_salt: hash_and_salt.salt,
        bio: format!("Default bio"),
        role_id: default_user_role,
    };
    let user = storage_specific_create_new_user(
        db_pool,
        new_user
    );

    Ok(user)
}

pub fn is_username_available(username: String,  db_pool: &db_manager::DbPool,) -> bool {
    let username_available = storage_specific_is_username_available(db_pool, username, );
    username_available
}

pub fn get_by_username(db_pool: &db_manager::DbPool, username: String) -> Option<User> {
    let mut client = db_manager::get_client(db_pool.clone());
    let query = "SELECT id FROM public.users WHERE username = $1";
    let result = client.query_opt(query, &[&username]).unwrap();
     match result {
        Some(row) => {
            let id = row.get("id");
            let user = get_user_by_user_id(db_pool, id).unwrap();
        },
        None => {
            return None;
        }
    }; 
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

fn storage_specific_is_username_available( db_pool: &db_manager::DbPool, username: String, ) -> bool {
    let mut client = db_manager::get_client(db_pool.clone());

    let row = client.query_one(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as exists",
        &[&username]
    ).unwrap();
    
    let exists: bool = row.get("exists");

    let is_username_available = !exists;
    is_username_available
}

fn storage_specific_create_new_user(
    db_pool: &db_manager::DbPool,
    new_user: User,
) -> User {
    storage_specific_add_user( db_pool, new_user.clone(),);

    new_user
}

fn storage_specific_add_user( db_pool: &db_manager::DbPool, user: User,) {
    let mut client = db_manager::get_client(db_pool.clone());
    let query = "INSERT INTO public.users (public_inapp_id, username, password_salt, password_hash, bio, role_id) VALUES ($1, $2, $3, $4, $5 )";
    client.execute(query, &[&user.id, &user.username, &user.password_hash, &user.password_salt, &user.bio, &user.role_id]).unwrap();
}

fn storage_specific_get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Option<User> {
    let mut client = db_manager::get_client(db_pool.clone());
    let result =   client.query_opt("SELECT public_inapp_id, username, role_id, bio, password_hash, password_salt FROM public.users WHERE public_inapp_id = $1", &[&user_id]).unwrap();
    match result {
        Some(row) => {
            let public_inapp_id = row.get("public_inapp_id");
        }
        None => {
            return None
        }
    }
}
