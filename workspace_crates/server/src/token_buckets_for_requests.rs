use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Instant};

pub type GlobalBucket = Arc<Mutex<TokenBucket>>;
pub type IpBuckets = Arc<Mutex<HashMap<IpAddr, TokenBucket>>>;

#[derive(Debug)]
pub struct TokenBucket {
    pub tokens: f64,
    pub capacity: f64,
    pub refill_rate_per_millis: f64,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate_per_millis: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate_per_millis,
            last_refill: Instant::now(),
        }
    }

    pub fn try_to_consume_one_token(&mut self) -> bool {
        let amount: f64 = 1.0;
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last_refill).as_millis() as f64;
        self.tokens = (self.tokens + elapsed_ms * self.refill_rate_per_millis).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }
}