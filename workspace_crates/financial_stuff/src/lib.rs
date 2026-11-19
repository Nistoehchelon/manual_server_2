use anyhow::{Context, anyhow};
use cryptocurrencies_stuff::{
    U256,
    atomic_unit::{AtomicUnit, CurrencyConfig},
    currencies::{CurrencyAtomicUnit, LTC, XMR, XNO},
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, Clone)]
pub struct ReferenceCurrency {
    pub name: String,
    pub amount: String,
}

/// Parses a decimal string (e.g., "12.34") into a U256 scaled by 10^decimals.
fn parse_decimal_to_u256(s: &str, decimals: u32) -> anyhow::Result<U256> {
    let s = s.trim();
    if s.is_empty() || s.starts_with('-') {
        return Err(anyhow!("Invalid non-negative decimal string: '{}'", s));
    }
    let s = s.strip_prefix('+').unwrap_or(s);

    let (whole_str, frac_str) = match s.split_once('.') {
        Some((w, f)) => (w, f),
        None => (s, ""),
    };

    if frac_str.contains('.') {
        return Err(anyhow!("Multiple decimal points in '{}'", s));
    }

    let whole = if whole_str.is_empty() {
        U256::ZERO
    } else {
        U256::from_str_radix(whole_str, 10).context("Invalid integer part")?
    };

    let scale = U256::from(10_u8).pow(U256::from(decimals));
    let mut frac_scaled = U256::ZERO;

    if !frac_str.is_empty() {
        let decimals_usize = decimals as usize;
        // Take up to `decimals` digits and right-pad with zeros to match the scale
        let truncated: String = frac_str.chars().take(decimals_usize).collect();
        if !truncated.chars().all(|c| c.is_ascii_digit()) {
            return Err(anyhow!("Invalid fractional part in '{}'", s));
        }
        let padded = format!("{:0<width$}", truncated, width = decimals_usize);
        frac_scaled = U256::from_str_radix(&padded, 10).context("Invalid fractional digits")?;
    }

    whole
        .checked_mul(scale)
        .and_then(|w| w.checked_add(frac_scaled))
        .ok_or_else(|| anyhow!("Decimal overflow while parsing '{}'", s))
}

pub async fn get_subscription_cost_usd(db_pool: sqlx::PgPool) -> anyhow::Result<String> {
    let mut tx = db_pool.begin().await?;
    let s = custom_site_variables::get_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::SubscriptionCostPerMonthUsd,
    )
    .await?;
    tx.commit().await?;
    Ok(s.trim().to_string())
}
pub async fn get_username_color_cost_per_month_usd(
    db_pool: sqlx::PgPool,
) -> anyhow::Result<String> {
    let mut tx = db_pool.begin().await?;
    let s = custom_site_variables::get_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::UsernameColorCostPerMonthUsd,
    )
    .await?;
    tx.commit().await?;
    Ok(s.trim().to_string())
}

pub async fn convert_usd_price_to_ltc(
    db_pool: sqlx::PgPool,
    price_usd: &str,
) -> anyhow::Result<AtomicUnit<LTC>> {
    let mut tx = db_pool.begin().await?;
    let answer = get_by_usd::<LTC>(&mut tx, price_usd).await?;
    tx.commit().await?;
    Ok(answer)
}

pub async fn convert_usd_price_to_xmr(
    db_pool: sqlx::PgPool,
    price_usd: &str,
) -> anyhow::Result<AtomicUnit<XMR>> {
    let mut tx = db_pool.begin().await?;
    let answer = get_by_usd::<XMR>(&mut tx, price_usd).await?;
    tx.commit().await?;
    Ok(answer)
}

pub async fn convert_usd_price_to_xno(
    db_pool: sqlx::PgPool,
    price_usd: &str,
) -> anyhow::Result<AtomicUnit<XNO>> {
    let mut tx = db_pool.begin().await?;
    let answer = get_by_usd::<XNO>(&mut tx, price_usd).await?;
    tx.commit().await?;
    Ok(answer)
}

pub async fn get_by_usd<C: CurrencyConfig>(
    tx: &mut Transaction<'_, Postgres>,
    usd: &str,
) -> anyhow::Result<AtomicUnit<C>> {
    let var_name = match C::SYMBOL {
        "LTC" => custom_site_variables::CustomSiteVariableName::USDPerLTC,
        "XMR" => custom_site_variables::CustomSiteVariableName::USDPerXMR,
        "XNO" => custom_site_variables::CustomSiteVariableName::USDPerXNO,
        _ => return Err(anyhow!("Unsupported crypto")),
    };

    let coin_price_str = custom_site_variables::get_custom_site_variable(tx, var_name).await?;

    // Use 18 decimal places of fixed-point precision for USD inputs
    const USD_DECIMALS: u32 = 18;
    let usd_scaled = parse_decimal_to_u256(usd, USD_DECIMALS)?;
    let price_scaled = parse_decimal_to_u256(&coin_price_str, USD_DECIMALS)?;

    if price_scaled.is_zero() {
        return Err(anyhow!("Coin price cannot be zero"));
    }

    // Both `usd_scaled` and `price_scaled` share the same 10^18 factor, so it cancels out:
    // atomic = round((usd_scaled * ATOMIC_UNITS_PER_ONE_WHOLE) / price_scaled)
    let numerator = usd_scaled
        .checked_mul(C::ATOMIC_UNITS_PER_ONE_WHOLE.0)
        .ok_or_else(|| anyhow!("Overflow calculating atomic units"))?;

    let quotient = numerator / price_scaled;
    let remainder = numerator % price_scaled;

    // Round half-up: add 1 if remainder * 2 >= price_scaled
    let atomic = if remainder
        .checked_mul(U256::from(2_u8))
        .is_none_or(|r2| r2 >= price_scaled)
    {
        quotient
            .checked_add(U256::from(1_u8))
            .ok_or_else(|| anyhow!("Overflow rounding atomic units"))?
    } else {
        quotient
    };

    Ok(AtomicUnit::<C>::from_atomic(CurrencyAtomicUnit(atomic)))
}
