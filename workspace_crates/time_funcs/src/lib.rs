use anyhow::Context;
pub fn utc_now() -> anyhow::Result<u64> {
    let now = chrono::Utc::now().timestamp();
    let now = u64::try_from(now)?;
    Ok(now)
}

const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60; // i.e., 30 days

pub fn add_months_to_timestamp(timestamp: u64, months: u64) -> anyhow::Result<u64> {
    let added_seconds = months
        .checked_mul(SECONDS_PER_MONTH)
        .context("Months overflowed u64")?;

    timestamp
        .checked_add(added_seconds)
        .context("Timestamp overflowed u64")
}
