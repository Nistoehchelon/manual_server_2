use std::fmt::{self, Display};

use anyhow::bail;
use db_manager::DbPool;
use error_structs::InternalAppError;

pub enum DesignatedPurpose {
    Normal,
}
impl Display for DesignatedPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Normal => "normal",
        };
        write!(f, "{}", s)
    }
}

pub async fn get_or_create_token(
    executor: db_manager::DbPool,
    session_id: String,
    designated_purpose: DesignatedPurpose,
) -> anyhow::Result<String> {
    let mut token_id: Option<String> = None;
    match db_manager::sql_queries::get_token_id_by_session_id_and_designated_purpose(
        executor.clone(),
        session_id.clone(),
        designated_purpose.to_string().clone(),
    )
    .await
    {
        Ok(t_opt) => {
            if let Some(t) = t_opt {
                token_id = Some(t);
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("{e:#?}"));
        }
    }
    if let Some(ti) = token_id {
        return Ok(ti);
    }
    const LENGTH: usize = 33;
    let dp: String = designated_purpose.to_string();
    let token_id = id_generator::generate_random_alphanumeric_id(LENGTH);
    match db_manager::sql_queries::add_new_token(
        executor,
        token_id.clone(),
        dp.clone(),
        session_id.clone(),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            bail!("{e:#?}");
        }
    };

    Ok(token_id)
}

pub async fn validate_token(
    executor: DbPool,
    session_id: String,
    token_id: String,
) -> anyhow::Result<bool> {
    db_manager::sql_queries::validate_token(executor, session_id, token_id)
        .await
}
