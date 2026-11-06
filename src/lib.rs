//! SigmaOS-clock-global — world clock registry (Omarchy `elsewhen` parity).
//!
//! Tracks cities with UTC offsets and formats their local time from a
//! Unix timestamp. No external time crates: civil-from-days algorithm
//! (Howard Hinnant) keeps it deterministic and dependency-free.

/// Seconds per day.
const SECS_PER_DAY: i64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityClock {
    pub city: String,
    pub utc_offset_hours: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    /// Days ahead of UTC date (e.g. +1 means "tomorrow" relative to UTC).
    pub day_offset: i32,
}

impl LocalTime {
    /// 24-hour "HH:MM:SS".
    pub fn format_24h(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    /// 12-hour "h:MM AM/PM".
    pub fn format_12h(&self) -> String {
        let h = self.hour % 12;
        let h = if h == 0 { 12 } else { h };
        let ap = if self.hour < 12 { "AM" } else { "PM" };
        format!("{}:{:02} {}", h, self.minute, ap)
    }
}

/// Days-from-civil algorithm: returns (year, month, day) — not needed for
/// clock display, but kept private for future date support.
#[allow(dead_code)]
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

#[derive(Debug, Default)]
pub struct GlobalClockRegistry {
    cities: Vec<CityClock>,
}

impl GlobalClockRegistry {
    pub fn new() -> Self {
        Self { cities: Vec::new() }
    }

    pub fn add_city(&mut self, city: &str, offset: i8) {
        // clamp to valid UTC offset range -12..=+14
        let offset = offset.clamp(-12, 14);
        self.cities.push(CityClock {
            city: city.into(),
            utc_offset_hours: offset,
        });
    }

    pub fn city_count(&self) -> usize {
        self.cities.len()
    }

    /// Compute local time for a city at a given unix timestamp (seconds).
    pub fn local_time(&self, city: &str, unix_secs: i64) -> Option<LocalTime> {
        let c = self.cities.iter().find(|c| c.city == city)?;
        let shifted = unix_secs + (c.utc_offset_hours as i64) * 3600;
        // floor-mod for negative timestamps
        let day = shifted.div_euclid(SECS_PER_DAY);
        let secs = shifted.rem_euclid(SECS_PER_DAY);
        let _utc_day = unix_secs.div_euclid(SECS_PER_DAY);
        Some(LocalTime {
            hour: (secs / 3600) as u32,
            minute: ((secs % 3600) / 60) as u32,
            second: (secs % 60) as u32,
            day_offset: (day - unix_secs.div_euclid(SECS_PER_DAY)) as i32,
        })
    }

    /// All cities formatted as "City  HH:MM:SS (+offset)".
    pub fn snapshot(&self, unix_secs: i64) -> Vec<String> {
        self.cities
            .iter()
            .filter_map(|c| {
                self.local_time(&c.city, unix_secs).map(|t| {
                    format!(
                        "{}  {} ({:+})",
                        c.city,
                        t.format_24h(),
                        c.utc_offset_hours
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_clock_registry_aaa() {
        let mut reg = GlobalClockRegistry::new();
        reg.add_city("San Francisco", -8);
        reg.add_city("London", 0);
        reg.add_city("New Delhi", 5);
        assert_eq!(reg.city_count(), 3);
    }

    #[test]
    fn utc_midnight_stays_midnight() {
        let mut reg = GlobalClockRegistry::new();
        reg.add_city("UTC", 0);
        // 2024-01-01T00:00:00Z = 1704067200
        let t = reg.local_time("UTC", 1_704_067_200).unwrap();
        assert_eq!(t.format_24h(), "00:00:00");
        assert_eq!(t.day_offset, 0);
    }

    #[test]
    fn offsets_shift_time_and_day() {
        let mut reg = GlobalClockRegistry::new();
        reg.add_city("Tokyo", 9);
        reg.add_city("LA", -8);
        // same instant: 2024-01-01T00:00:00Z
        let tokyo = reg.local_time("Tokyo", 1_704_067_200).unwrap();
        let la = reg.local_time("LA", 1_704_067_200).unwrap();
        assert_eq!(tokyo.format_24h(), "09:00:00");
        assert_eq!(tokyo.day_offset, 0);
        assert_eq!(la.format_24h(), "16:00:00"); // previous day 16:00
        assert_eq!(la.day_offset, -1);
    }

    #[test]
    fn offset_clamped_to_valid_range() {
        let mut reg = GlobalClockRegistry::new();
        reg.add_city("Extreme", 99);
        // offset clamped to +14
        let t = reg.local_time("Extreme", 0).unwrap();
        assert_eq!(t.hour, 14);
    }

    #[test]
    fn format_12h() {
        let t = LocalTime { hour: 15, minute: 4, second: 9, day_offset: 0 };
        assert_eq!(t.format_12h(), "3:04 PM");
        let t2 = LocalTime { hour: 0, minute: 30, second: 0, day_offset: 0 };
        assert_eq!(t2.format_12h(), "12:30 AM");
    }
}
