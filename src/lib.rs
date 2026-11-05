#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityClock {
    pub city: String,
    pub utc_offset_hours: i8,
}

#[derive(Debug, Default)]
pub struct GlobalClockRegistry {
    cities: Vec<CityClock>,
}

impl GlobalClockRegistry {
    pub fn new() -> Self { Self { cities: Vec::new() } }
    pub fn add_city(&mut self, city: &str, offset: i8) {
        self.cities.push(CityClock { city: city.into(), utc_offset_hours: offset });
    }
    pub fn city_count(&self) -> usize { self.cities.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_global_clock_registry_aaa() {
        // Arrange
        let mut reg = GlobalClockRegistry::new();
        // Act
        reg.add_city("San Francisco", -8);
        reg.add_city("London", 0);
        reg.add_city("New Delhi", 5);
        // Assert
        assert_eq!(reg.city_count(), 3);
    }
}
