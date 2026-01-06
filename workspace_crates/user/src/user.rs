use db_manager::{DbError, DbParams};
use error_structs::PublicError;

use crate::user_structs_and_so_on::User;

pub async fn get_user_by_user_id(
    user_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Option<User>, DbError> {
    let user_opt = storage_specific_get_user_by_user_id(user_id, db_pool).await?;
    return Ok(user_opt);
}

pub struct CheckingError {
    pub message: String,
}
pub async fn try_to_create_new_user(
    db_pool: &db_manager::DbPool,
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
        public_id: generate_user_id(),
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
    db_pool: &db_manager::DbPool,
) -> Result<bool, DbError> {
    let bool_whether_username_available =
        storage_specific_is_username_available(db_pool, username).await?;
    Ok(bool_whether_username_available)
}

pub async fn get_by_username(
    username: String,
    db_pool: &db_manager::DbPool,
) -> Result<Option<User>, DbError> {
    let mut client = db_manager::get_client(&db_pool).await?;
    let transaction = match client.transaction().await {
        Err(e) => {
            return Err(DbError::new(e));
        }
        Ok(tx) => tx,
    };
    let query = "SELECT public_id FROM public.users WHERE username = $1";
    let result = match transaction.query_opt(query, &[&username]).await {
        Ok(res) => res,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };
    match result {
        Some(row) => {
            let id: String = row.get("public_id");
            let params: DbParams = &[&id];
            let (user_query, params) = sql_queries::get_user_by_user_id(params);
            let user_row = match transaction.query_one(user_query, params).await {
                Ok(ur) => ur,
                Err(e) => {
                    return Err(DbError::new(e));
                }
            };
            return Ok(Some(User::from(user_row)));
        }
        None => {
            return Ok(None);
        }
    };
}

pub async fn get_by_username_and_password(
    username: String,
    password: String,
    db_pool: &db_manager::DbPool,
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
    db_pool: &db_manager::DbPool,
    username: String,
) -> Result<bool, DbError> {
    let client = db_manager::get_client(&db_pool).await?;

    let row = match client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) as exists",
            &[&username],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };

    let exists: bool = row.get("exists");

    let is_username_available = !exists;
    Ok(is_username_available)
}

async fn storage_specific_create_new_user(
    db_pool: &db_manager::DbPool,
    new_user: User,
) -> Result<User, DbError> {
    storage_specific_add_user(db_pool, new_user.clone()).await?;

    Ok(new_user)
}

async fn storage_specific_add_user(
    db_pool: &db_manager::DbPool,
    user: User,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    let client = db_manager::get_client(&db_pool).await?;
    let params: DbParams =  &[
            &user.public_id,
            &user.username,
            &user.password_hash,
            &user.password_salt,
            &user.bio,
        ];
    let (query, params) = sql_queries::add_user(params);
    let _modified_rows = db_manager::execute(
        &client,
        query,
        params,
    )
    .await?;
    Ok(db_manager::DbSuccess {
        message: format!("Done!"),
    })
}

async fn storage_specific_get_user_by_user_id(
    user_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Option<User>, db_manager::DbError> {
    let client = db_manager::get_client(&db_pool).await?;
    let params: DbParams = &[&user_id];
    let (query, params) = sql_queries::get_user_by_user_id(params);
    let result = db_manager::query_opt(&client, query, params).await?;
    Ok(result.map(User::from))
}
