use anyhow::anyhow;
use cryptocurrencies_stuff::{
    atomic_unit::{AtomicUnit, CryptocurrencyConfig},
    currencies::{LTC, XMR},
};
use sqlx::{Postgres, Transaction};

pub const SUBSCRIPTION_COST_USD: f64 = 1.0;

pub async fn get_subscription_cost_usd(db_pool: sqlx::PgPool) -> anyhow::Result<f64> {
    let mut tx = db_pool.begin().await?;
    let s = custom_site_variables::get_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::SubscriptionCostUsd,
    )
    .await?;
    tx.commit().await?;
    let cost = s.parse::<f64>()?;
    Ok(cost)
}

pub async fn convert_usd_price_to_ltc(db_pool: sqlx::PgPool, price: f64) -> anyhow::Result<f64> {
    let mut tx = db_pool.begin().await?;
    let answer = get_by_usd::<LTC>(&mut tx, price)
        .await?
        .to_string_trimmed()
        .parse::<f64>()?;
    tx.commit().await?;
    Ok(answer)
}
pub async fn convert_usd_price_to_xmr(db_pool: sqlx::PgPool, price: f64) -> anyhow::Result<f64> {
    let mut tx = db_pool.begin().await?;
    let answer = get_by_usd::<XMR>(&mut tx, price)
        .await?
        .to_string_trimmed()
        .parse::<f64>()?;
    tx.commit().await?;
    Ok(answer)
}

pub async fn get_by_usd<C: CryptocurrencyConfig>(
    tx: &mut Transaction<'_, Postgres>,
    usd: f64,
) -> anyhow::Result<AtomicUnit<C>> {
    let coin_price_usd = match C::SYMBOL {
        "LTC" => {
            let usd_per_ltc = custom_site_variables::get_custom_site_variable(
                tx,
                custom_site_variables::CustomSiteVariableName::USDPerLTC,
            )
            .await?;
            let usd_per_ltc: f64 = usd_per_ltc.parse()?;
            usd_per_ltc
        }
        "XMR" => {
            let usd_per_xmr = custom_site_variables::get_custom_site_variable(
                tx,
                custom_site_variables::CustomSiteVariableName::USDPerXMR,
            )
            .await?;
            let usd_per_xmr: f64 = usd_per_xmr.parse()?;
            usd_per_xmr
        }
        _ => return Err(anyhow!("Unsupported crypto")),
    };

    let atomic = ((usd / coin_price_usd) * C::ATOMIC_PER_UNIT as f64).round() as u64;

    Ok(AtomicUnit::<C>::from_atomic(atomic))
}
