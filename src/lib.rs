//! SigmaOS Global World Clock (inspired by Elsewhen).
//!
//! Principles: Command-Query Separation, Single Responsibility, Encapsulation,
//! Arrange-Act-Assert, Defensive Programming, Zero-Copy observation.

pub mod timezone;
pub mod clock;

pub use timezone::{TimeZoneOffset, TimeZoneError};
pub use clock::{CityClock, WorldClockManager};
