use cryptocurrencies_stuff::currencies::XMRAtomicUnit;
use custom_site_variables::set_custom_site_variable;
use error_structs::SomeSortOfError;
use strum::{Display, EnumString};

pub const X_API_TOKEN: &str = "x-api-token";

pub struct WebhookToken {
    pub primary_key: i64,
    pub token: String,
    pub designated_purpose: String,
}

#[derive(Debug, EnumString, Display)]
pub enum WebhookEndpoint {
    #[strum(serialize = "payment_success")]
    PaymentSuccess,
    #[strum(serialize = "payment_status_update")]
    PaymentStatusUpdate,
}

pub async fn get_endpoint_url(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    secret_prefix: String,
    endpoint: WebhookEndpoint,
) -> Result<String, SomeSortOfError> {
    let webhook_root = get_webhook_root(tx, secret_prefix).await?;
    Ok(format!("{webhook_root}/{}", endpoint))
}

pub async fn get_webhook_root(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    secret_prefix: String,
) -> Result<String, SomeSortOfError> {
    use custom_site_variables::CustomSiteVariableName;
    let Some(website_root) = custom_site_variables::find_custom_site_variable(
        tx,
        CustomSiteVariableName::WebsiteRootUrl,
    )
    .await?
    else {
        return Err(SomeSortOfError::new("..."))?;
    };

    Ok(format!("{website_root}/{secret_prefix}/webhook"))
}

pub async fn validate_token(db_pool: sqlx::PgPool, token: String) -> Result<bool, SomeSortOfError> {
    let token = sqlx::query_as!(
        WebhookToken,
        "SELECT primary_key, token, designated_purpose FROM webhook_tokens WHERE token = $1",
        token
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(|e| {
        println!("Could not validate token");
        SomeSortOfError::new(format!("Token err, {e}"))
    })?;
    Ok(token.is_some())
}

pub async fn get_token(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, SomeSortOfError> {
    let token = sqlx::query_as!(
        WebhookToken,
        "SELECT primary_key, token, designated_purpose FROM webhook_tokens LIMIT 1",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(token.token)
}
pub async fn set_website_fund_xmr(
    db_pool: sqlx::PgPool,
    amount: XMRAtomicUnit,
) -> Result<(), SomeSortOfError> {
    let amount_visual = amount.to_string_trimmed();
    let amount_atomic = amount.as_atomic().to_string();
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    set_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::WebsiteFundXMRBalanceAtomicUnits,
        amount_atomic,
    )
    .await?;
    set_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::WebsiteFundXMRBalanceVisual,
        amount_visual,
    )
    .await?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(())
}
