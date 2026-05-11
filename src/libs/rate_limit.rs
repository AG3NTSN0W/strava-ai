use chrono::{DateTime, Timelike, Utc};

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub limit_15min: u32,
    pub limit_daily: u32,
    pub usage_15min: u32,
    pub usage_daily: u32,
    pub updated_at: DateTime<Utc>,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            limit_15min: 0,
            limit_daily: 0,
            usage_15min: 0,
            usage_daily: 0,
            updated_at: DateTime::<Utc>::MIN_UTC,
        }
    }
}

impl RateLimit {
    /// Returns true if we have budget remaining in both the 15-min and daily windows.
    /// Strava's 15-min window resets at :00, :15, :30, :45 each hour.
    /// The daily window resets at midnight UTC.
    pub fn has_budget(&self) -> bool {
        if self.limit_15min == 0 && self.limit_daily == 0 {
            return true; // no data yet, assume OK
        }

        let now = Utc::now();

        // Check if the 15-min window has rolled over since last update
        let current_window_start = now
            .with_minute((now.minute() / 15) * 15)
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now);
        let fifteen_min_reset = self.updated_at < current_window_start;

        // Check if the daily window has rolled over (midnight UTC)
        let today_midnight = now
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now);
        let daily_reset = self.updated_at < today_midnight;

        let within_15min = fifteen_min_reset || self.usage_15min < self.limit_15min * 90 / 100;
        let within_daily = daily_reset || self.usage_daily < self.limit_daily * 90 / 100;

        if !within_15min || !within_daily {
            log::warn!(
                "Rate limit exhausted: 15min {}/{}, daily {}/{}",
                self.usage_15min, self.limit_15min, self.usage_daily, self.limit_daily
            );
        }

        within_15min && within_daily
    }
}
