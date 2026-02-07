use config_reader::Config;
use http_stuff::http_response::standalone_response;
use std::net::IpAddr;
use std::time::Duration;
use std::time::Instant;

use crate::token_buckets_for_requests::GlobalBuckets;
use crate::token_buckets_for_requests::IpBuckets;
use crate::token_buckets_for_requests::TimeBucket;
use crate::token_buckets_for_requests::TokenAndTimeBucketsAndLastSeen;
use crate::token_buckets_for_requests::TokenBucket;
use http_stuff::http_response::StatusCode;

pub async fn check_and_consume_buckets(
    now: Instant,
    peer_ip: &IpAddr,
    global_bucket: &GlobalBuckets,
    ip_buckets: &IpBuckets,
    Config {
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis,
        per_ip_time_credits_capacity,
        per_ip_time_credits_refill_rate_per_millis,
        ..
    }: Config,
) -> Result<(), Vec<u8>> {
    let mut ip_buckets = ip_buckets.lock().await;
    // I guess it makes sense to remove old entries.... ?
    if ip_buckets.len() > 1000 {
        let cutoff = now - Duration::from_secs(150);
        ip_buckets.retain(|_, entry| entry.last_seen >= cutoff);
    }
    let ip_bucket = ip_buckets
        .entry(*peer_ip)
        .or_insert_with(|| TokenAndTimeBucketsAndLastSeen {
            token_bucket: TokenBucket::new(
                per_ip_token_bucket_capacity,
                per_ip_token_bucket_refill_rate_per_millis,
            ),
            time_bucket: TimeBucket::new(
                per_ip_time_credits_capacity,
                per_ip_time_credits_refill_rate_per_millis,
            ),
            last_seen: Instant::now(),
        });

    ip_bucket.last_seen = now;

    let mut reject = false;
    if !ip_bucket.token_bucket.try_to_consume_one_token() {
        reject = true;
        dbg!(format!("Ip {} is out of tokens", peer_ip));
    }
    if !ip_bucket.time_bucket.has_time() {
        reject = true;
        dbg!(format!("Ip {} is out of time credits", peer_ip));
    }
    if reject {
        let response = standalone_response(
            StatusCode::Code429TooManyRequests,
            "429 Too Many Requests 😈".to_string(),
        );
        return Err(response);
    }

    let mut global_bucket = global_bucket.lock().await;
    let mut reject = false;
    if !global_bucket.token_bucket.try_to_consume_one_token() {
        reject = true;
        dbg!("Out of global tokens");
    }
    if !global_bucket.time_bucket.has_time() {
        reject = true;
        dbg!("Out of global time");
    }
    if reject {
        let response = standalone_response(
            StatusCode::Code503ServiceUnavailable,
            "503 Service Unavailable 🫨".to_string(),
        );
        return Err(response);
    }

    Ok(())
}
