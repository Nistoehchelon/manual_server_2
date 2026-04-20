use error_structs::SomeSortOfError;

pub struct WebhooksToken {
    pub primary_key: i64,
    pub token: String,
    pub designated_purpose: String,
}

pub struct XMRAmount(u64);

impl XMRAmount {
//    const ATOMIC_PER_XMR: u64 = 1_000_000_000_000;

    pub fn from_atomic(a: u64) -> Self {
        Self(a)
    }

    pub fn as_atomic(&self) -> u64 {
        self.0
    }
}
pub async fn validate_token(db_pool: sqlx::PgPool, token: String) -> Result<bool, SomeSortOfError> {
    let token = sqlx::query_as!(WebhooksToken, "SELECT primary_key, token, designated_purpose FROM webhook_tokens WHERE token = $1", token).fetch_optional(&db_pool).await.map_err(SomeSortOfError::new)?;
    Ok(token.is_some())

}

pub async fn add_website_fund_xmr(db_pool: sqlx::PgPool, amount: XMRAmount) -> Result<(), SomeSortOfError> {
    let name = "website_fund_xmr";
    let add_amount = amount.as_atomic().to_string();
    let current_amount = sqlx::query!("SELECT value FROM custom_site_variables WHERE name = $1", name).fetch_one(&db_pool).await.map_err(SomeSortOfError::new)?.value;
    let new_amount = {
        let add_amount = add_amount.parse::<i64>().map_err(SomeSortOfError::new)?;
        let current_amount = current_amount.parse::<i64>().map_err(SomeSortOfError::new)?;
        current_amount + add_amount
    }.to_string();

    sqlx::query!("UPDATE custom_site_variables SET value = $1 WHERE name = $2", new_amount, name).execute(&db_pool).await.map_err(SomeSortOfError::new)?;
    Ok(())
}