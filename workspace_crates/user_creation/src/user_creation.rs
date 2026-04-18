use error_structs::{PublicError, SomeSortOfError};
use password_hashing::PasswordHasherConfig;
use sqlx::PgPool;
use user::User;

pub async fn try_to_create_new_user(
    db_pool: PgPool,
    username: String,
    password: String,
    hasher_config: PasswordHasherConfig,
) -> Result<Result<User, PublicError>, SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;

    let user = match user::try_to_create_new_user(&mut tx, username, password, hasher_config)
        .await
        .map_err(SomeSortOfError::new)?
    {
        Err(e) => {
            return Ok(Err(e));
        }
        Ok(u) => u,
    };

    sqlx::query!(
        "INSERT INTO access_attributes (subject, subject_id, name) VALUES($1, $2, $3)",
        "user",
        user.id.clone(),
        "create_thread"
    )
    .execute(&mut *tx)
    .await
    .map_err(SomeSortOfError::new)?;

    // Test rollback if we enter this specific username, we test if this whole tx fails if we return before committing
    if user.username.clone() == "test_transaction_failure" {
        return Ok(Err(PublicError::new(format!(
            "testing transaction auto-rollback on error. This artificial error is triggered right after inserting the user but before tx.commit'ting. Thus the user {} should not be created due to auto-rollback. Try logging in. If it fails, this means this test has succeeded.",
            user.username
        ))));
    }

    tx.commit().await.map_err(SomeSortOfError::new)?;

    Ok(Ok(user))
}
