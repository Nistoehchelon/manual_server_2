use db_manager::{DbError, DbSuccess};
use error_structs::{ExpectedToGetButFailed, PublicError, SomeSortOfError};
use password_hashing::PasswordHasherConfig;
use sqlx::{Postgres, Transaction};

use crate::{ValidatedUsername, user_structs_and_so_on::User};

pub async fn get_user_by_user_id(
    db_pool: sqlx::PgPool,
    user_id: String,
) -> Result<User, error_structs::ExpectedToGetButFailed> {
    match find_user_by_user_id(db_pool, user_id)
        .await
        .map_err(ExpectedToGetButFailed::new)?
    {
        Some(u) => Ok(u),
        None => Err(ExpectedToGetButFailed::new("")),
    }
}
pub async fn find_user_by_user_id(
    db_pool: sqlx::PgPool,
    user_id: String,
) -> Result<Option<User>, DbError> {
    let user_opt = storage_specific_get_user_by_user_id(user_id, db_pool).await?;
    Ok(user_opt)
}

pub struct CheckingError {
    pub message: String,
}
pub async fn try_to_create_new_user(
    tx: &mut Transaction<'_, Postgres>,
    username: ValidatedUsername,
    password_hash: String,
) -> Result<Result<User, PublicError>, SomeSortOfError> {
    let new_user = User {
        primary_key: 0,
        id: id_generator_unique_in_db::generate_unique_user_id(tx)
            .await
            .map_err(SomeSortOfError::new)?,
        username: username.to_string(),
        password_hash,
    };
    let user = storage_specific_create_new_user(tx, new_user)
        .await
        .map_err(SomeSortOfError::new)?;

    Ok(Ok(user))
}

pub async fn set_user_data(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    data_name: &str,
    data_value: &str,
) -> Result<(), SomeSortOfError> {
    sqlx::query!(
        r#"
        INSERT INTO user_data (
            user_id,
            data_name,
            data_value
        )
        VALUES (
            $1,
            $2,
            $3
        )
        ON CONFLICT (user_id, data_name)
        DO UPDATE SET
            data_value = EXCLUDED.data_value
        "#,
        user_id,
        data_name,
        data_value
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(())
}

#[derive(Debug)]
pub struct UserData {
    pub primary_key: i64,
    pub user_id: String,
    pub data_name: String,
    pub data_value: String,
}

pub async fn find_user_data_by_name_and_value(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    value: &str,
) -> Result<Vec<UserData>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UserData,
        r#"
        SELECT
            primary_key,
            user_id,
            data_name,
            data_value
        FROM user_data
        WHERE data_name = $1
          AND data_value = $2
        "#,
        name,
        value
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows)
}

pub async fn get_user_data(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
    data_name: &str,
) -> Result<Option<String>, SomeSortOfError> {
    let row = sqlx::query!(
        r#"
        SELECT data_value
        FROM user_data
        WHERE user_id = $1
          AND data_name = $2
        "#,
        user_id,
        data_name
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(row.map(|r| r.data_value))
}

pub async fn is_username_available(
    username: String,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<bool, DbError> {
    let bool_whether_username_available =
        storage_specific_is_username_available(tx, username).await?;
    Ok(bool_whether_username_available)
}

pub async fn get_by_username(
    username: String,
    db_pool: sqlx::PgPool,
) -> Result<Option<User>, DbError> {
    let mut transaction = db_pool.begin().await.map_err(DbError::new)?;

    let result = match sqlx::query!(
        r#"SELECT id FROM public.users WHERE username = $1"#,
        username
    )
    .fetch_optional(&mut *transaction)
    .await
    {
        Err(e) => {
            return Err(DbError::new(e));
        }
        Ok(opt) => opt,
    };

    match result {
        Some(row) => {
            let id: String = row.id;
            let user_row = match sqlx::query_as!(User,r#"SELECT primary_key, id, username, password_hash FROM public.users WHERE id = $1"#, id).fetch_one(&mut *transaction).await {
                Ok(ur) => ur,
                Err(e) => {
                    return Err(DbError::new(e));
                }
            };
            Ok(Some(user_row))
        }
        None => Ok(None),
    }
}

pub async fn find_by_username_and_password(
    username: String,
    password: String,
    db_pool: sqlx::PgPool,
    hasher_config: PasswordHasherConfig,
) -> Result<Option<User>, SomeSortOfError> {
    let user = get_by_username(username.clone(), db_pool)
        .await
        .map_err(SomeSortOfError::new)?;
    let user = match user {
        Some(u) => u,
        None => return Ok(None),
    };
    if !password_hashing::verify_password(password.clone(), user.password_hash.clone())
        .map_err(SomeSortOfError::new)?
    {
        dbg!("Failed password hash verification");
        dbg!(password.clone());
        dbg!(user.clone().password_hash);
        let tried_hash = password_hashing::hash_password(password.clone(), hasher_config)
            .map_err(SomeSortOfError::new)?;

        dbg!(tried_hash);
        return Ok(None);
    }
    Ok(Some(user))
}

async fn storage_specific_is_username_available(
    tx: &mut Transaction<'_, Postgres>,
    username: String,
) -> Result<bool, DbError> {
    let exists = match sqlx::query!(
        r#"SELECT username FROM users WHERE username = $1"#,
        username,
    )
    .fetch_optional(&mut **tx)
    .await
    {
        Ok(opt) => opt.is_some(),
        Err(e) => {
            return Err(DbError::new(e));
        }
    };

    let is_username_available = !exists;
    Ok(is_username_available)
}

async fn storage_specific_create_new_user(
    tx: &mut Transaction<'_, Postgres>,
    new_user: User,
) -> Result<User, DbError> {
    storage_specific_add_user(&mut *tx, new_user.clone()).await?;

    Ok(new_user)
}

async fn storage_specific_add_user(
    tx: &mut Transaction<'_, Postgres>,
    user: User,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    match sqlx::query!(
        r#"INSERT INTO users (id, username, password_hash) VALUES ($1,$2,$3)"#,
        user.id,
        user.username,
        user.password_hash,
    )
    .execute(&mut **tx)
    .await
    {
        Ok(_) => Ok(DbSuccess::new("User created")),
        Err(e) => Err(DbError::new(e)),
    }
}

async fn storage_specific_get_user_by_user_id(
    user_id: String,
    db_pool: sqlx::PgPool,
) -> Result<Option<User>, db_manager::DbError> {
    match sqlx::query_as!(
        User,
        r#"SELECT primary_key, id, username, password_hash FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&db_pool)
    .await
    {
        Ok(opt) => Ok(opt),
        Err(e) => Err(DbError::new(e)),
    }
}
