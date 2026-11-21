use anyhow::Context;
use chrono::{DateTime, Datelike, Utc};
pub fn utc_now_secs() -> anyhow::Result<u64> {
    let now = chrono::Utc::now().timestamp();
    let now = u64::try_from(now)?;
    Ok(now)
}
pub fn utc_now() -> DateTime<Utc> {
    
    chrono::Utc::now()
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

pub fn format_utc_custom(dt: chrono::DateTime<chrono::Utc>) -> String {
    let day = dt.day();
    let suffix = match day {
        11..=13 => "th",
        _ => match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };

    dt.format(&format!("{}{} %B %Y %H:%M:%S UTC+0", day, suffix))
        .to_string()
}
