//! Timezone offset representation and calculation.
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum TimeZoneError {
    InvalidFormat(String),
    OffsetOutOfRange(i32),
}

impl fmt::Display for TimeZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeZoneError::InvalidFormat(s) => write!(f, "Invalid timezone format: {s}"),
            TimeZoneError::OffsetOutOfRange(m) => write!(f, "Offset out of range (-720..=840 minutes): {m}"),
        }
    }
}

/// Offset from UTC in total minutes (-720 min to +840 min, i.e. UTC-12 to UTC+14)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeZoneOffset(pub i32);

impl TimeZoneOffset {
    pub const UTC: Self = Self(0);

    pub fn from_hours_minutes(hours: i32, minutes: i32) -> Result<Self, TimeZoneError> {
        let total = if hours < 0 {
            hours * 60 - minutes.abs()
        } else {
            hours * 60 + minutes.abs()
        };
        Self::from_minutes(total)
    }

    pub fn from_minutes(minutes: i32) -> Result<Self, TimeZoneError> {
        if !(-720..=840).contains(&minutes) {
            return Err(TimeZoneError::OffsetOutOfRange(minutes));
        }
        Ok(Self(minutes))
    }

    pub fn total_minutes(self) -> i32 {
        self.0
    }

    pub fn difference_from(self, other: Self) -> i32 {
        self.0 - other.0
    }
}

impl fmt::Display for TimeZoneOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 >= 0 { '+' } else { '-' };
        let abs = self.0.abs();
        let hours = abs / 60;
        let mins = abs % 60;
        write!(f, "UTC{}{:02}:{:02}", sign, hours, mins)
    }
}

impl FromStr for TimeZoneOffset {
    type Err = TimeZoneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "Z" || s == "UTC" {
            return Ok(Self::UTC);
        }
        let stripped = s.strip_prefix("UTC").unwrap_or(s);
        let sign = match stripped.chars().next() {
            Some('+') => 1,
            Some('-') => -1,
            _ => return Err(TimeZoneError::InvalidFormat(s.to_string())),
        };
        let rest = &stripped[1..];
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.is_empty() || parts.len() > 2 {
            return Err(TimeZoneError::InvalidFormat(s.to_string()));
        }
        let hours = parts[0].parse::<i32>().map_err(|_| TimeZoneError::InvalidFormat(s.to_string()))?;
        let minutes = if parts.len() == 2 {
            parts[1].parse::<i32>().map_err(|_| TimeZoneError::InvalidFormat(s.to_string()))?
        } else {
            0
        };
        Self::from_hours_minutes(sign * hours, minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_timezone_offsets() {
        // Arrange & Act
        let ist = TimeZoneOffset::from_hours_minutes(5, 30).unwrap();
        let est = TimeZoneOffset::from_hours_minutes(-5, 0).unwrap();

        // Assert
        assert_eq!(ist.total_minutes(), 330);
        assert_eq!(est.total_minutes(), -300);
        assert_eq!(ist.to_string(), "UTC+05:30");
        assert_eq!(est.to_string(), "UTC-05:00");
    }

    #[test]
    fn test_difference_between_timezones() {
        // Arrange
        let ist = TimeZoneOffset::from_hours_minutes(5, 30).unwrap();
        let utc = TimeZoneOffset::UTC;

        // Act & Assert
        assert_eq!(ist.difference_from(utc), 330);
    }

    #[test]
    fn test_parse_timezone_string() {
        // Arrange & Act
        let parsed: TimeZoneOffset = "UTC+05:30".parse().unwrap();
        let parsed_z: TimeZoneOffset = "Z".parse().unwrap();

        // Assert
        assert_eq!(parsed.total_minutes(), 330);
        assert_eq!(parsed_z, TimeZoneOffset::UTC);
    }

    #[test]
    fn test_out_of_range_rejected() {
        // Arrange & Act
        let res = TimeZoneOffset::from_minutes(900); // 15 hours > 14 hours

        // Assert
        assert!(matches!(res, Err(TimeZoneError::OffsetOutOfRange(900))));
    }
}
