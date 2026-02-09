use config_reader::Config;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct TokenAndTimeBuckets {
    pub token_bucket: TokenBucket,
    pub time_bucket: TimeBucket,
}
pub struct TokenAndTimeBucketsAndLastSeen {
    pub token_bucket: TokenBucket,
    pub time_bucket: TimeBucket,
    pub last_seen: Instant,
}
impl TokenAndTimeBucketsAndLastSeen {
    pub fn default_with_settings(server_config: Config) -> Self {
        Self {
            token_bucket: TokenBucket::new(
                server_config.per_ip_token_bucket_capacity,
                server_config.per_ip_token_bucket_refill_rate_per_millis,
            ),
            time_bucket: TimeBucket::new(
                server_config.per_ip_time_credits_capacity,
                server_config.per_ip_time_credits_refill_rate_per_millis,
            ),
            last_seen: Instant::now(),
        }
    }
}

pub type GlobalBuckets = Arc<Mutex<TokenAndTimeBuckets>>;
pub type IpBuckets = Arc<Mutex<HashMap<IpAddr, TokenAndTimeBucketsAndLastSeen>>>;

#[derive(Debug)]
pub struct TokenBucket {
    pub tokens: f64,
    pub tokens_capacity: f64,
    pub tokens_refill_rate_per_millis: f64,

    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(tokens_capacity: f64, tokens_refill_rate_per_millis: f64) -> Self {
        Self {
            tokens: tokens_capacity,
            tokens_capacity,
            tokens_refill_rate_per_millis,
            last_refill: Instant::now(),
        }
    }

    pub fn try_to_consume_one_token(&mut self) -> bool {
        let now = Instant::now();
        let amount: f64 = 1.0;
        let elapsed_ms = now.duration_since(self.last_refill).as_millis() as f64;
        self.tokens = (self.tokens + elapsed_ms * self.tokens_refill_rate_per_millis)
            .min(self.tokens_capacity);
        self.last_refill = now;

        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct TimeBucket {
    pub time_credits: Duration,
    pub time_credits_capacity: Duration,
    pub time_credits_refill_rate_per_millis: Duration,
    pub last_refill: Instant,
}

impl TimeBucket {
    pub fn new(
        time_credits_capacity: Duration,
        time_credits_refill_rate_per_millis: Duration,
    ) -> Self {
        Self {
            last_refill: Instant::now(),

            time_credits: time_credits_capacity,
            time_credits_capacity,
            time_credits_refill_rate_per_millis,
        }
    }

    pub fn consume_time(&mut self, t: Duration) {
        self.refill();
        self.time_credits = self.time_credits.saturating_sub(t);
    }
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed.is_zero() {
            return;
        }

        let millis: u32 = match elapsed.as_millis().try_into() {
            Ok(num) => num,
            Err(e) => {
                dbg!(e);
                return;
            }
        };

        let refill_amount = self.time_credits_refill_rate_per_millis * millis;

        self.time_credits = (self.time_credits + refill_amount).min(self.time_credits_capacity);

        self.last_refill = now;
    }
    pub fn has_time(&mut self) -> bool {
        self.refill();
        self.time_credits > Duration::ZERO
    }
}
