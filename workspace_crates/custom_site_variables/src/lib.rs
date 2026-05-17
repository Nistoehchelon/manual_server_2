use error_structs::SomeSortOfError;
use sqlx::{Postgres, Transaction};
use strum::{Display, EnumString};

#[derive(Debug, EnumString, Display)]
pub enum CustomSiteVariableName {
    #[strum(serialize = "payment_server_url")]
    PaymentServerUrl,
    #[strum(serialize = "payment_server_token")]
    PaymentServerToken,
    #[strum(serialize = "subscription_cost_usd")]
    SubscriptionCostUsd,
    #[strum(serialize = "usd_per_ltc")]
    USDPerLTC,
    #[strum(serialize = "usd_per_xmr")]
    USDPerXMR,
    #[strum(serialize = "website_fund_xmr_balance_atomic_units")]
    WebsiteFundXMRBalanceAtomicUnits,
    #[strum(serialize = "website_fund_xmr_balance_visual")]
    WebsiteFundXMRBalanceVisual,
    #[strum(serialize = "website_root_url")]
    WebsiteRootUrl,
}

pub async fn set_custom_site_variable(
    tx: &mut Transaction<'_, Postgres>,
    var_name: CustomSiteVariableName,
    var_value: String,
) -> Result<(), SomeSortOfError> {
    sqlx::query!(
        r#"
        INSERT INTO custom_site_variables (name, value)
        VALUES ($1, $2)
        ON CONFLICT (name)
        DO UPDATE SET value = EXCLUDED.value
        "#,
        var_name.to_string(),
        var_value,
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(())
}
pub async fn find_custom_site_variable(
    tx: &mut Transaction<'_, Postgres>,
    var_name: CustomSiteVariableName,
) -> Result<Option<String>, SomeSortOfError> {
    let row = sqlx::query!(
        r#"SELECT value FROM custom_site_variables WHERE name = $1"#,
        var_name.to_string()
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;
    if let Some(row) = row {
        let val = row.value;
        return Ok(Some(val));
    }
    Err(SomeSortOfError::new(format!(
        "Could not find custom_site_setting {var_name}",
    )))
}
