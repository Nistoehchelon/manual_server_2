use error_structs::SomeSortOfError;

pub struct WebhookToken {
    pub primary_key: i64,
    pub token: String,
    pub designated_purpose: String,
}

pub struct XMRAtomicUnit(u64);

impl XMRAtomicUnit {
    const ATOMIC_PER_XMR: u64 = 1_000_000_000_000;
    // count number of zeroes in case XMR increases the size from 12 to 16 or something and I will be too lazy to change manually
    const PRECISION: usize = Self::ATOMIC_PER_XMR.ilog10() as usize;

    pub fn from_atomic(a: u64) -> Self {
        Self(a)
    }

    pub fn as_atomic(&self) -> u64 {
        self.0
    }
    pub fn to_xmr_string_trimmed(&self) -> String {
        let whole = self.0 / Self::ATOMIC_PER_XMR;
        let fractional = self.0 % Self::ATOMIC_PER_XMR;

        if fractional == 0 {
            return whole.to_string();
        }

        let frac_str = format!("{:0width$}", fractional, width = Self::PRECISION)
            .trim_end_matches('0')
            .to_string();

        format!("{}.{}", whole, frac_str)
    }
}
pub async fn validate_token(db_pool: sqlx::PgPool, token: String) -> Result<bool, SomeSortOfError> {
    let token = sqlx::query_as!(
        WebhookToken,
        "SELECT primary_key, token, designated_purpose FROM webhook_tokens WHERE token = $1",
        token
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(token.is_some())
}

pub async fn set_website_fund_xmr(
    db_pool: sqlx::PgPool,
    amount: XMRAtomicUnit,
) -> Result<(), SomeSortOfError> {
    let atomic_var_name = "website_fund_xmr_balance_atomic_units";
    let visual_var_name = "website_fund_xmr_balance_visual";
    let amount_visual = amount.to_xmr_string_trimmed();
    let amount_atomic = amount.as_atomic().to_string();
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    sqlx::query!(
        r#"
        INSERT INTO custom_site_variables (name, value)
        VALUES ($1, $2)
        ON CONFLICT (name)
        DO UPDATE SET value = EXCLUDED.value
        "#,
        atomic_var_name,
        amount_atomic,
    )
    .execute(&mut *tx)
    .await
    .map_err(SomeSortOfError::new)?;
    sqlx::query!(
        r#"
        INSERT INTO custom_site_variables (name, value)
        VALUES ($1, $2)
        ON CONFLICT (name)
        DO UPDATE SET value = EXCLUDED.value
        "#,
        visual_var_name,
        amount_visual,
        
    )
    .execute(&mut *tx)
    .await
    .map_err(SomeSortOfError::new)?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(())
}
