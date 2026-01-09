use db_manager::{DbError, DbSuccess};
use error_structs::{ExpectedToGetButFailedError, PublicError};

use crate::user_structs_and_so_on::User;


pub async fn get_user_by_id(db_pool: &sqlx::PgPool, user_id: String) -> Result<Result<User, error_structs::ExpectedToGetButFailedError>, DbError> {
    match find_user_by_user_id(user_id, db_pool).await? {
        Some(u) => {
            return Ok(Ok(u));
        },
        None => {
            return Ok(Err(ExpectedToGetButFailedError::new("Expected user to exist")));
        },
    }
}
pub async fn find_user_by_user_id(
    user_id: String,
    db_pool: &sqlx::PgPool,
) -> Result<Option<User>, DbError> {
    let user_opt = storage_specific_get_user_by_user_id(user_id, db_pool).await?;
    return Ok(user_opt);
}

pub struct CheckingError {
    pub message: String,
}
pub async fn try_to_create_new_user(
    db_pool: &sqlx::PgPool,
    username: String,
    password: String,
) -> Result<Result<User, PublicError>, DbError> {
    if username.clone().trim().is_empty() || password.clone().trim().is_empty() {
        return Ok(Err(PublicError::new(format!(
            "Please enter username and password or the world will be destroyed!"
        ))));
    }

    let username_available = is_username_available(username.clone(), db_pool).await?;

    if !username_available {
        return Ok(Err(PublicError::new(format!(
            "The username is... taken alredy!!! 😈"
        ))));
    }

    let hash_and_salt = password_hashing::hash_password_generate_salt(password);
    let new_user = User {
        primary_key: 0,
        id: generate_user_id(),
        username,
        password_hash: hash_and_salt.hash,
        password_salt: hash_and_salt.salt,
        bio: format!("Default bio"),
    };
    let user = storage_specific_create_new_user(db_pool, new_user).await?;

    Ok(Ok(user))
}

pub async fn is_username_available(
    username: String,
    db_pool: &sqlx::PgPool,
) -> Result<bool, DbError> {
    let bool_whether_username_available =
        storage_specific_is_username_available(db_pool, username).await?;
    Ok(bool_whether_username_available)
}

pub async fn get_by_username(
    username: String,
    db_pool: &sqlx::PgPool,
) -> Result<Option<User>, DbError> {
    let mut transaction = db_pool.begin().await.map_err(DbError::new)?; 

   let result = match sqlx::query!(r#"SELECT id FROM public.users WHERE username = $1"#, username)
   .fetch_optional(&mut *transaction).await {
        Err(e) =>  {
            return Err(DbError::new(e));
        },
        Ok(opt) => opt,
   };

    match result {
        Some(row) => {
            let id: String = row.id;
            let user_row = match sqlx::query_as!(User,r#"SELECT primary_key, id, username, password_hash, password_salt, bio FROM public.users WHERE id = $1"#, id).fetch_one(&mut *transaction).await {
                Ok(ur) => ur,
                Err(e) => {
                    return Err(DbError::new(e));
                }
            };
            return Ok(Some(user_row));
        }
        None => {
            return Ok(None);
        }
    };
}

pub async fn get_by_username_and_password(
    username: String,
    password: String,
    db_pool: &sqlx::PgPool,
) -> Result<Option<User>, DbError> {
    let user = get_by_username(username.clone(), db_pool).await?;
    let user = match user {
        Some(u) => u,
        None => return Ok(None),
    };
    if password_hashing::verify_password(
        password.clone(),
        user.password_hash.clone(),
        user.password_salt.clone(),
    ) != true
    {
        dbg!("Failed password hash verification");
        dbg!(password.clone());
        dbg!(user.clone().password_hash);
        dbg!(user.clone().password_salt);
        let tried_hash =
            password_hashing::hash_password(password.clone(), user.password_salt.clone());

        dbg!(tried_hash);
        return Ok(None);
    }
    Ok(Some(user))
}

fn generate_user_id() -> String {
    id_generator::generate_id()
}

async fn storage_specific_is_username_available(
    db_pool: &sqlx::PgPool,
    username: String,
) -> Result<bool, DbError> {

    let exists = match sqlx::query!(
            r#"SELECT username FROM users WHERE username = $1"#,
            username,
        ).fetch_optional(db_pool)
        .await {
            Ok(opt) => {
                opt.is_some()
            },
            Err(e) => {
                return Err(DbError::new(e));
            }
        };


    let is_username_available = !exists;
    Ok(is_username_available)
}

async fn storage_specific_create_new_user(
    db_pool: &sqlx::PgPool,
    new_user: User,
) -> Result<User, DbError> {
    storage_specific_add_user(db_pool, new_user.clone()).await?;

    Ok(new_user)
}

async fn storage_specific_add_user(
    db_pool: &sqlx::PgPool,
    user: User,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    match sqlx::query!(r#"INSERT INTO users (id, username, password_hash, password_salt, bio) VALUES ($1,$2,$3,$4,$5)"#, user.id, user.username, user.password_hash, user.password_salt, user.bio).execute(db_pool).await {
        Ok(_) => {
            return Ok(DbSuccess::new("User created"));
        },
        Err(e) => {
            return Err(DbError::new(e));
        }
    }
}
 

async fn storage_specific_get_user_by_user_id(
    user_id: String,
    db_pool: &sqlx::PgPool,
) -> Result<Option<User>, db_manager::DbError> {
    match sqlx::query_as!(User, r#"SELECT primary_key, id, username, password_hash, password_salt, bio FROM users WHERE id = $1"#, user_id).fetch_optional(db_pool).await {
        Ok(opt) => {
            return Ok(opt);
        },
        Err(e)=>{
            return Err(DbError::new(e));
        }
    }
}
