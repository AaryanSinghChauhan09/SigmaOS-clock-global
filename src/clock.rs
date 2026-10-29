//! World clock manager and city time calculations.
use crate::timezone::TimeZoneOffset;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityClock {
    pub city: String,
    pub country: String,
    pub offset: TimeZoneOffset,
    pub is_dst_active: bool,
}

impl CityClock {
    pub fn new(city: impl Into<String>, country: impl Into<String>, offset: TimeZoneOffset) -> Self {
        Self {
            city: city.into(),
            country: country.into(),
            offset,
            is_dst_active: false,
        }
    }

    pub fn effective_offset(&self) -> TimeZoneOffset {
        if self.is_dst_active {
            TimeZoneOffset::from_minutes(self.offset.total_minutes() + 60)
                .unwrap_or(self.offset)
        } else {
            self.offset
        }
    }

    /// Calculate local civil time given UTC epoch seconds (hh, mm, ss).
    pub fn calculate_local_time(&self, utc_epoch_secs: u64) -> (u32, u32, u32) {
        let total_secs = utc_epoch_secs as i64 + (self.effective_offset().total_minutes() as i64 * 60);
        let rem_secs = total_secs.rem_euclid(86400);
        let hours = (rem_secs / 3600) as u32;
        let mins = ((rem_secs % 3600) / 60) as u32;
        let secs = (rem_secs % 60) as u32;
        (hours, mins, secs)
    }
}

pub struct WorldClockManager {
    cities: BTreeMap<String, CityClock>,
}

impl WorldClockManager {
    pub fn new() -> Self {
        Self {
            cities: BTreeMap::new(),
        }
    }

    // ── Queries (Command-Query Separation) ──────────────────────────────────
    pub fn city_count(&self) -> usize {
        self.cities.len()
    }

    pub fn get_city(&self, city: &str) -> Option<&CityClock> {
        self.cities.get(city)
    }

    pub fn list_cities(&self) -> Vec<&CityClock> {
        self.cities.values().collect()
    }

    pub fn time_diff_minutes(&self, city_a: &str, city_b: &str) -> Option<i32> {
        let a = self.cities.get(city_a)?;
        let b = self.cities.get(city_b)?;
        Some(a.effective_offset().difference_from(b.effective_offset()))
    }

    // ── Commands (Command-Query Separation) ──────────────────────────────────
    pub fn add_city(&mut self, clock: CityClock) {
        self.cities.insert(clock.city.clone(), clock);
    }

    pub fn remove_city(&mut self, city: &str) -> Option<CityClock> {
        self.cities.remove(city)
    }

    pub fn set_dst(&mut self, city: &str, active: bool) -> Result<(), String> {
        match self.cities.get_mut(city) {
            Some(c) => {
                c.is_dst_active = active;
                Ok(())
            }
            None => Err(format!("City {city} not found")),
        }
    }
}

impl Default for WorldClockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query_city_clock() {
        // Arrange
        let mut mgr = WorldClockManager::new();
        let tokyo = CityClock::new("Tokyo", "Japan", TimeZoneOffset::from_hours_minutes(9, 0).unwrap());

        // Act
        mgr.add_city(tokyo);

        // Assert
        assert_eq!(mgr.city_count(), 1);
        assert_eq!(mgr.get_city("Tokyo").unwrap().country, "Japan");
    }

    #[test]
    fn test_time_difference_between_cities() {
        // Arrange
        let mut mgr = WorldClockManager::new();
        mgr.add_city(CityClock::new("London", "UK", TimeZoneOffset::UTC));
        mgr.add_city(CityClock::new("New York", "USA", TimeZoneOffset::from_hours_minutes(-5, 0).unwrap()));

        // Act
        let diff = mgr.time_diff_minutes("London", "New York").unwrap();

        // Assert
        assert_eq!(diff, 300); // 5 hours = 300 minutes
    }

    #[test]
    fn test_calculate_local_time_at_utc_midnight() {
        // Arrange
        let ist = CityClock::new("New Delhi", "India", TimeZoneOffset::from_hours_minutes(5, 30).unwrap());
        let utc_midnight = 0; // 00:00:00 UTC

        // Act
        let (h, m, s) = ist.calculate_local_time(utc_midnight);

        // Assert
        assert_eq!((h, m, s), (5, 30, 0));
    }

    #[test]
    fn test_dst_adjustment() {
        // Arrange
        let mut london = CityClock::new("London", "UK", TimeZoneOffset::UTC);

        // Act
        london.is_dst_active = true;

        // Assert
        assert_eq!(london.effective_offset().total_minutes(), 60);
    }
}
