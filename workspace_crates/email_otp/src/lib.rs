
use std::fmt;

use chrono::Utc;
use sqlx::{FromRow, Postgres, Transaction};

const OTP_TTL_SECS: i64 = 15 * 60;

#[derive(Debug, FromRow)]
pub struct PendingUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug)]
pub enum ConsumeEmailOtpError {
    CodeNotFound,
    AlreadyConsumed,
    Expired,
    Database(sqlx::Error),
}

impl fmt::Display for ConsumeEmailOtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeNotFound => write!(f, "otp code not found"),
            Self::AlreadyConsumed => write!(f, "otp code already consumed"),
            Self::Expired => write!(f, "otp code expired"),
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for ConsumeEmailOtpError {}

impl From<sqlx::Error> for ConsumeEmailOtpError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

pub async fn consume_email_otp(
   tx: &mut Transaction<'_, Postgres>,
    code: &str,
) -> Result<PendingUser, ConsumeEmailOtpError> {

    let otp = sqlx::query!(
        r#"
        SELECT
            primary_key,
            consumed_at,
            created_at
        FROM email_otps
        WHERE code = $1
        FOR UPDATE
        "#,
        code
    )
    .fetch_optional(&mut **tx)
    .await?;

    let otp = match otp {
        Some(otp) => otp,
        None => return Err(ConsumeEmailOtpError::CodeNotFound),
    };

    if otp.consumed_at.is_some() {
        return Err(ConsumeEmailOtpError::AlreadyConsumed);
    }

    let expires_at = otp.created_at.unix_timestamp() + OTP_TTL_SECS;

    if  expires_at < Utc::now().timestamp() {
        return Err(ConsumeEmailOtpError::Expired);
    }

    let pending_user = sqlx::query_as!(
        PendingUser,
        r#"
        SELECT
            email,
            username,
            password_hash
        FROM email_otp_pending_users
        WHERE email_otp_primary_key = $1
        "#,
        otp.primary_key
    )
    .fetch_optional(&mut **tx)
    .await?;

    let pending_user = match pending_user {
        Some(user) => user,
        None => return Err(ConsumeEmailOtpError::CodeNotFound),
    };

    sqlx::query!(
        r#"
        UPDATE email_otps
        SET consumed_at = NOW()
        WHERE primary_key = $1
        "#,
        otp.primary_key
    )
    .execute(&mut **tx)
    .await?;


    Ok(pending_user)
}


#[derive(Debug)]
pub struct CreateEmailOtpInput {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug)]
pub enum CreateEmailOtpError {
    Database(sqlx::Error),
}

impl From<sqlx::Error> for CreateEmailOtpError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

#[derive(Debug)]
pub struct CreatedEmailOtp {
    pub primary_key: i64,
    pub code: String,
}

pub async fn create_email_otp(
    pool: sqlx::PgPool,
    input: CreateEmailOtpInput,
) -> Result<CreatedEmailOtp, CreateEmailOtpError> {
    let mut tx = pool.begin().await?;

    // Generate random 32-character code
    let code: String = id_generator::generate_random_alphanumeric_id(32);

    // Create OTP row
    let otp = sqlx::query!(
        r#"
        INSERT INTO email_otps (
            code,
            created_at
        )
        VALUES (
            $1,
            NOW()
        )
        RETURNING primary_key
        "#,
        code
    )
    .fetch_one(&mut *tx)
    .await?;

    // Store pending user data
    sqlx::query!(
        r#"
        INSERT INTO email_otp_pending_users (
            email_otp_primary_key,
            email,
            username,
            password_hash
        )
        VALUES (
            $1,
            $2,
            $3,
            $4
        )
        "#,
        otp.primary_key,
        input.email,
        input.username,
        input.password_hash
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(CreatedEmailOtp {
        primary_key: otp.primary_key,
        code,
    })
}