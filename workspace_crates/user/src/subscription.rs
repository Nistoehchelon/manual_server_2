use error_structs::SomeSortOfError;
use sqlx::{Postgres, Transaction};

use crate::user;

pub async fn user_has_a_subscription(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &str,
) -> Result<bool, SomeSortOfError> {
    let data = user::get_user_data(tx, user_id, "subscription_until").await?;
    match data {
        Some(s) => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let curr_unix_ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(SomeSortOfError::new)?
                .as_secs();
            let sub_until_ts: u64 = s.parse().map_err(SomeSortOfError::new)?;
            if sub_until_ts > curr_unix_ts {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}
