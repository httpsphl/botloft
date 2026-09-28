//! Courier configuration, from `config.toml` or set by tests.

use std::time::Duration;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct CourierSettings {
    pub poll_interval: Duration,
    /// How long a delivery may stay `sending` before it counts as lost.
    pub lease: Duration,
    pub max_attempts: u32,
    pub retry_backoff_initial: Duration,
    pub retry_backoff_max: Duration,
}

impl CourierSettings {
    pub fn from_config(config: &Config) -> Self {
        let courier = &config.courier;
        Self {
            poll_interval: Duration::from_millis(courier.poll_interval_ms.max(1)),
            lease: Duration::from_millis(courier.lease_ms),
            max_attempts: courier.max_attempts.max(1),
            retry_backoff_initial: Duration::from_millis(courier.retry_backoff_initial_ms),
            retry_backoff_max: Duration::from_millis(courier.retry_backoff_max_ms),
        }
    }

    /// Wait before the next try after `attempts` failures:
    /// `initial * 2^(attempts - 1)`, capped at the max (spec 9.1).
    pub fn retry_delay(&self, attempts: u32) -> Duration {
        let factor = 1u32
            .checked_shl(attempts.saturating_sub(1).min(20))
            .unwrap_or(u32::MAX);
        self.retry_backoff_initial
            .saturating_mul(factor)
            .min(self.retry_backoff_max.max(self.retry_backoff_initial))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_double_up_to_the_cap() {
        let settings = CourierSettings::from_config(&Config::default());
        let delays: Vec<_> = (1..=8).map(|n| settings.retry_delay(n).as_secs()).collect();
        assert_eq!(delays, [2, 4, 8, 16, 32, 64, 120, 120]);
        assert_eq!(settings.retry_delay(u32::MAX).as_secs(), 120);
    }
}
