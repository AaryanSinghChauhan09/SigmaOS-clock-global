# SigmaOS Global Clock

A Rust library for representing fixed offsets from UTC and calculating clock
readings from caller supplied Unix epoch seconds. It includes a small city
manager keyed by city name and a manually controlled daylight saving offset.

## Library surface

- `TimeZoneOffset` stores an offset in whole minutes and formats it as
  `UTC±HH:MM`.
- `FromStr` accepts `UTC`, `Z`, and signed hour or hour-and-minute offsets.
- `CityClock::calculate_local_time` applies the effective offset to epoch
  seconds and returns `(hour, minute, second)` in a 24-hour day.
- `WorldClockManager` adds, removes, looks up, and lists city clocks and
  compares their effective offsets.

```rust
use sigmaos_clock_global::{CityClock, TimeZoneOffset, WorldClockManager};

let india = TimeZoneOffset::from_hours_minutes(5, 30).unwrap();
let mut clocks = WorldClockManager::new();
clocks.add_city(CityClock::new("New Delhi", "India", india));
let local_time = clocks.get_city("New Delhi").unwrap().calculate_local_time(0);
assert_eq!(local_time, (5, 30, 0));
```

## Time and DST limits

Offsets are fixed values; this crate does not use IANA timezone data, resolve
historical transitions, or determine daylight saving rules. `is_dst_active`
manually adds one hour when the resulting offset is valid. Callers needing civil
time for a real location should supply authoritative timezone rules and account
for date-dependent transitions themselves. Epoch input is an unsigned seconds
count, and the result intentionally contains only time of day, not a date.

## Development

This crate has no third party dependencies. Run `cargo check` to compile the
library and `cargo test` to run its unit tests.
