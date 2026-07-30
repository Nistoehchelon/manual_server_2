use cryptocurrencies_stuff::{
    atomic_unit::{AtomicUnit, CryptocurrencyConfig},
    currencies::{LTC, XMR},
};
use error_structs::SomeSortOfError;
use sqlx::{Postgres, Transaction};

pub const SUBSCRIPTION_COST_USD: f64 = 1.0;

pub async fn get_subscription_cost_usd(db_pool: sqlx::PgPool) -> Result<f64, SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    let Some(s) = custom_site_variables::find_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::SubscriptionCostUsd,
    )
    .await?
    else {
        return Err(SomeSortOfError::new(
            "Didn't find subscription_cost_usd site var",
        ));
    };
    tx.commit().await.map_err(SomeSortOfError::new)?;
    let cost = s.parse::<f64>().map_err(SomeSortOfError::new)?;
    Ok(cost)
}

pub async fn convert_usd_price_to_ltc(
    db_pool: sqlx::PgPool,
    price: f64,
) -> Result<f64, SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    let answer = get_by_usd::<LTC>(&mut tx, price)
        .await?
        .to_string_trimmed()
        .parse::<f64>()
        .map_err(SomeSortOfError::new)?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(answer)
}
pub async fn convert_usd_price_to_xmr(
    db_pool: sqlx::PgPool,
    price: f64,
) -> Result<f64, SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    let answer = get_by_usd::<XMR>(&mut tx, price)
        .await?
        .to_string_trimmed()
        .parse::<f64>()
        .map_err(SomeSortOfError::new)?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(answer)
}

pub async fn get_by_usd<C: CryptocurrencyConfig>(
    tx: &mut Transaction<'_, Postgres>,
    usd: f64,
) -> Result<AtomicUnit<C>, SomeSortOfError> {
    let coin_price_usd = match C::SYMBOL {
        "LTC" => {
            let Some(usd_per_ltc) = custom_site_variables::find_custom_site_variable(
                tx,
                custom_site_variables::CustomSiteVariableName::USDPerLTC,
            )
            .await
            .map_err(SomeSortOfError::new)?
            else {
                return Err(SomeSortOfError::new("Could not get usd per ltc site var"));
            };
            let usd_per_ltc: f64 = usd_per_ltc.parse().map_err(SomeSortOfError::new)?;
            usd_per_ltc
        }
        "XMR" => {
            let Some(usd_per_xmr) = custom_site_variables::find_custom_site_variable(
                tx,
                custom_site_variables::CustomSiteVariableName::USDPerXMR,
            )
            .await
            .map_err(SomeSortOfError::new)?
            else {
                return Err(SomeSortOfError::new("Could not get usd per xmr site var"));
            };
            let usd_per_xmr: f64 = usd_per_xmr.parse().map_err(SomeSortOfError::new)?;
            usd_per_xmr
        }
        _ => return Err(SomeSortOfError::new("Unsupported crypto")),
    };

    let atomic = ((usd / coin_price_usd) * C::ATOMIC_PER_UNIT as f64).round() as u64;

    Ok(AtomicUnit::<C>::from_atomic(atomic))
}
