use http_stuff::http_response::standalone_response;
use std::net::IpAddr;

use crate::token_buckets_for_requests::GlobalBucket;
use crate::token_buckets_for_requests::IpBuckets;
use crate::token_buckets_for_requests::TokenBucket;
use http_stuff::http_response::StatusCode;

pub async fn check_and_consume_buckets(
    peer_ip: &IpAddr,
    global_bucket: &GlobalBucket,
    ip_buckets: &IpBuckets,
    per_ip_token_bucket_capacity: f64,
    per_ip_token_bucket_refill_rate_per_millis: f64,
) -> Result<(), Vec<u8>> {
    // Check global bucket
    if !global_bucket.lock().await.try_to_consume_one_token() {
        let response = standalone_response(
            StatusCode::ServiceUnavailable503,
            "503 Service Unavailable 🫨".to_string(),
        );
        return Err(response);
    }

    // Check per-IP bucket
    let mut ip_buckets_guard = ip_buckets.lock().await;
    let ip_bucket = ip_buckets_guard.entry(*peer_ip).or_insert_with(|| {
        TokenBucket::new(
            per_ip_token_bucket_capacity,
            per_ip_token_bucket_refill_rate_per_millis,
        )
    });

    if !ip_bucket.try_to_consume_one_token() {
        let response = standalone_response(
            StatusCode::TooManyRequests429,
            "429 Too Many Requests 😈".to_string(),
        );
        return Err(response);
    }

    Ok(())
}
