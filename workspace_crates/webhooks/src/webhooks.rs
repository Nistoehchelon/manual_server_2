use cryptocurrencies_stuff::atomic_unit::XMRAtomicUnit;
use custom_site_variables::set_custom_site_variable;
use error_structs::SomeSortOfError;

pub struct WebhookToken {
    pub primary_key: i64,
    pub token: String,
    pub designated_purpose: String,
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
    let amount_visual = amount.to_string_trimmed();
    let amount_atomic = amount.as_atomic().to_string();
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    set_custom_site_variable(&mut tx, custom_site_variables::CustomSiteVariableName::WebsiteFundXMRBalanceAtomicUnits, amount_atomic).await?;
    set_custom_site_variable(&mut tx, custom_site_variables::CustomSiteVariableName::WebsiteFundXMRBalanceVisual, amount_visual).await?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(())
}
