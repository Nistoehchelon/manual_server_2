use error_structs::{SomeSortOfError};


pub async fn try_to_create_new_user(
    db_pool: sqlx::PgPool,
    username: String,
    password: String,
    hasher_config: password_hashing::PasswordHasherConfig,
) -> Result<Result<user::User, error_structs::PublicError>, error_structs::SomeSortOfError> {
    let user = match user::try_to_create_new_user(db_pool.clone(), username, password, hasher_config).await.map_err(SomeSortOfError::new)? {
        Err(e) => {
            return Ok(Err(e));
        },
        Ok(u) => u,
    };


    sqlx::query!(
        "INSERT INTO access_attributes (subject, subject_id, name) VALUES($1, $2, $3)",
        "user",
        user.id.clone(),
        "create_thread"
    )
    .execute(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(Ok(user))
}