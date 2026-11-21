use anyhow::Context;
use sqlx::{PgConnection, Postgres, Transaction};
use std::collections::HashMap;
use strum::IntoEnumIterator;

pub struct CustomSiteVariableEntry {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, strum::EnumString, strum::Display, strum::EnumIter)]
pub enum CustomSiteVariableName {
    #[strum(serialize = "payment_server_url")]
    PaymentServerUrl,
    #[strum(serialize = "payment_server_token")]
    PaymentServerToken,
    #[strum(serialize = "subscription_cost_per_month_usd")]
    SubscriptionCostPerMonthUsd,
    #[strum(serialize = "username_color_cost_per_month_usd")]
    UsernameColorCostPerMonthUsd,
    #[strum(serialize = "usd_per_ltc")]
    USDPerLTC,
    #[strum(serialize = "usd_per_xmr")]
    USDPerXMR,
    #[strum(serialize = "usd_per_xno")]
    USDPerXNO,
    #[strum(serialize = "website_fund_xmr_balance_atomic_units")]
    WebsiteFundXMRBalanceAtomicUnits,
    #[strum(serialize = "website_fund_xmr_balance_visual")]
    WebsiteFundXMRBalanceVisual,
    #[strum(serialize = "website_root_url")]
    WebsiteRootUrl,
    #[strum(serialize = "allowed_username_colors")]
    AllowedUsernameColor,
    #[strum(serialize = "shop_seller_pub_key_url")]
    ShopSellerPubKeyUrl,
    #[strum(serialize = "shop_seller_name")]
    ShopSellerName,

}

pub async fn set_custom_site_variable(
    tx: &mut Transaction<'_, Postgres>,
    var_name: CustomSiteVariableName,
    var_value: String,
) -> anyhow::Result<()> {
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
    .await?;
    Ok(())
}
pub async fn get_custom_site_variable(
    conn: &mut PgConnection,
    var_name: CustomSiteVariableName,
) -> anyhow::Result<String> {
    let result = sqlx::query!(
        r#"SELECT value FROM custom_site_variables WHERE name = $1"#,
        var_name.to_string()
    )
    .fetch_one(&mut *conn)
    .await.context(format!("Could not get custom site variable {var_name}"))?
    .value;
    Ok(result)
}

pub async fn find_custom_site_variable(
    tx: &mut PgConnection,
    var_name: CustomSiteVariableName,
) -> anyhow::Result<Option<String>> {
    let row = sqlx::query!(
        r#"SELECT value FROM custom_site_variables WHERE name = $1"#,
        var_name.to_string(),
    )
    .fetch_optional(&mut *tx)
    .await.context("Db error")?;
    Ok(row.map(|r| r.value))
}
pub async fn save_custom_site_variables_from_form(
    tx: &mut Transaction<'_, Postgres>,
    form: &HashMap<String, String>,
) -> anyhow::Result<()> {
    for variant in CustomSiteVariableName::iter() {
        let name = variant.to_string();
        const NOT_SET_PLACEHOLDER: &str = "[not set]";
        match form.get(&name).map(String::as_str) {
            // Non-empty → upsert
            Some(v) if !v.is_empty() && v != NOT_SET_PLACEHOLDER => {
                sqlx::query!(
                    r#"
                    INSERT INTO custom_site_variables (name, value)
                    VALUES ($1, $2)
                    ON CONFLICT (name) DO UPDATE SET value = EXCLUDED.value
                    "#,
                    name,
                    v,
                )
                .execute(&mut **tx)
                .await?;
            }
            // Missing or empty → "not set" → remove the row
            _ => {
                sqlx::query!("DELETE FROM custom_site_variables WHERE name = $1", name,)
                    .execute(&mut **tx)
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn get_all_custom_site_variables(
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<CustomSiteVariableEntry>> {
    let rows = sqlx::query!("SELECT name, value FROM custom_site_variables")
        .fetch_all(&mut *conn)
        .await?;

    let mut map: HashMap<String, String> = rows.into_iter().map(|r| (r.name, r.value)).collect();

    let entries: Vec<CustomSiteVariableEntry> = CustomSiteVariableName::iter()
        .map(|v| {
            let name = v.to_string();
            let value = map.remove(&name);
            CustomSiteVariableEntry { name, value }
        })
        .collect();

    Ok(entries)
}
