
use crate::{get_query_str, user_structs_and_so_on::User};
use db_manager::DbFailureError;

pub struct UserError {
    pub message: String,
}


pub fn get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Result<Option<User>, DbFailureError> {
 match storage_specific_get_user_by_user_id(user_id, db_pool) {
        Ok(opt) => Ok(opt),
        Err(_) => Err(DbFailureError {message: format!("Error getting user")}),
    }
}

pub fn try_to_create_new_user(
    db_pool: &db_manager::DbPool,
    username: String,
    password: String,
) -> Result<Result<User, UserError>, DbFailureError> {
    if username.clone().trim().is_empty() || password.clone().trim().is_empty() {
        return Err(DbFailureError { message: "Please enter username and password!!".to_string()} );
    }

    let username_available = match is_username_available(username.clone(), db_pool)  {
        Ok(b) => b,
        Err(_) => return Err(DbFailureError { message: format!("Username is unavailable. 😒😒😒") }),
    };
    if !username_available{
        return Err(DbFailureError{message: format!("Username {username} is taken already!")});
    }

    let hash_and_salt = password_hashing::hash_password(password);
    let default_user_role = 1;
    let new_user = User {
        internal_id: 0,
        public_inapp_id: generate_user_id(),
        username,
        password_hash: hash_and_salt.hash,
        password_salt: hash_and_salt.salt,
        bio: format!("Default bio"),
        role_id: default_user_role,
    };
    let user = match storage_specific_create_new_user(
        db_pool,
        new_user
    ) {
        Ok(user) => user,
        Err(_) => return Err(DbFailureError { message: format!("Error creating user") }),
    };

    Ok(Ok(user))
}

pub fn is_username_available(username: String,  db_pool: &db_manager::DbPool,) -> Result<bool, DbFailureError> {
    let username_available = storage_specific_is_username_available(db_pool, username, )?;
    Ok(username_available)
}

pub fn get_by_username( username: String,db_pool: &db_manager::DbPool,) -> Result<Option<User>, DbFailureError> {
    let mut client = db_manager::get_client(db_pool.clone());
    let mut transaction = match client.transaction() {
        Err(_) => return Err(DbFailureError { message: format!("Could not start tx") }),
        Ok(tx) => tx,
    };
    let query = "SELECT id FROM public.users WHERE username = $1";
    let result = match transaction.query_opt(query, &[&username]) {
        Ok(res) => res,
        Err(_) => return Err(DbFailureError { message: format!("Query error") }),
    };
     match result {
        Some(row) => {
            let id: String = row.get("id");
            let user_query = get_query_str(crate::QueryStr::GetUserById);
            let user_row = match transaction.query_one(user_query, &[&id]) {
                Ok(ur) => ur,
                Err(_) => return Err(DbFailureError { message: format!("Err") }),
            };
            return Ok(Some(User::from(user_row)));
        },
        None => {
            return Ok(None);
        }
    }; 
}

pub fn get_by_username_and_password(username: String, password: String, db_pool: &db_manager::DbPool,) -> Result<Option<User>, String> {
    let user = match get_by_username(username.clone(), db_pool) {
        Ok(user) => user,
        Err(_) => {
            return Err("Err".to_string());
        }
    };
    let user = match user {
        Some(u) => u,
        None => return Ok(None),
    };
    if password_hashing::verify_password(password, user.password_hash.clone(), user.password_salt.clone())
        != true
    {
        return Err(
            "User found but hash of input password and existing password do not match".to_string(),
        );
    }
    Ok(Some(user))
}

fn generate_user_id() -> String {
    id_generator::generate_id()
}



fn storage_specific_is_username_available( db_pool: &db_manager::DbPool, username: String, ) -> Result<bool, DbFailureError> {
    let mut client = db_manager::get_client(db_pool.clone());

    let row = match client.query_one(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as exists",
        &[&username]
    ) {
        Ok(r) => r,
        Err(_) => return Err(DbFailureError {message: format!("...")}),
    };
    
    let exists: bool = row.get("exists");

    let is_username_available = !exists;
    Ok(is_username_available)
}

fn storage_specific_create_new_user(
    db_pool: &db_manager::DbPool,
    new_user: User,
) -> Result<User, DbFailureError> {
    match storage_specific_add_user( db_pool, new_user.clone(),) {
        Ok(_) => {},
        Err(_) => return Err(DbFailureError {message: "Could not create user".to_string()}),
    }

    Ok(new_user)
}

fn storage_specific_add_user( db_pool: &db_manager::DbPool, user: User,)-> Result<db_manager::DbSuccess, db_manager::DbError> {
    let client = db_manager::get_client(db_pool.clone());
    let query = "INSERT INTO public.users (public_inapp_id, username, password_salt, password_hash, bio, role_id) VALUES ($1, $2, $3, $4, $5 )";
    let _modified_rows = db_manager::execute(client, query, &[&user.public_inapp_id, &user.username, &user.password_hash, &user.password_salt, &user.bio, &user.role_id])?;
    Ok(db_manager::DbSuccess { message: format!("Done!") })
}

fn storage_specific_get_user_by_user_id(user_id: String, db_pool: &db_manager::DbPool) -> Result<Option<User>, db_manager::DbError > {
    
    let client = db_manager::get_client(db_pool.clone());
    
    let result =   db_manager::query_opt(client, crate::get_query_str(crate::QueryStr::GetUserById), &[&user_id])?;
    Ok(result.map(User::from))
}
