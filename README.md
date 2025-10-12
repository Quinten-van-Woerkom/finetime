# Finetime: Accurate, flexible, and efficient time keeping
`finetime` is a Rust library for accurate, flexible, and efficient timekeeping, designed for applications where precision and performance are critical.
- **Accurate**: Supports exact arithmetic with attosecond-level precision over arbitrary time ranges, without sacrificing correctness or performance.
- **Flexible**: Built on Rust generics, `finetime` allows durations and time points to be expressed as integers or floats of any native Rust bitwidth, in any SI time unit, and using any time scale.
- **Efficient**: Represents time values as tick counts since an epoch, enabling compact storage and fast processing without conversion overhead.
- **Verified**: Key correctness properties have been formally proven using the [`Kani` model checker](https://model-checking.github.io/kani/), ensuring a high degree of reliability.
- **Portable**: The `finetime` library is fully `no_std`, such that it may be used even in bare metal environments.

With this fine degree of control and precision, `finetime` is suitable for all types of applications, from nanoseconds in embedded systems to femtoseconds in scientific computing, or picoseconds for precise orbit determination.

## Getting started
`finetime` requires the `cargo` build system for the Rust programming language to be present on your system. The library may be added as dependency for your Rust project by running `cargo add finetime`. Afterwards, it may be used directly by importing `finetime` into your Rust source code.

## Expressing time points
In `finetime`, time points are always bound to a specific timekeeping standard, indicated as `TimeScale`. One such example is Coordinated Universal Time (UTC). Time points may be constructed directly from some given datetime in the historic calendar:
```rust
use finetime::{UtcTime, Month};
let epoch = UtcTime::from_datetime(2025, Month::August, 3, 20, 25, 42).unwrap();
```
Note that constructing time points from datetimes may fail, because the given arguments do not form a valid time-of-day, or because the given date did not occur in the historic calendar: `finetime` makes this explicit. Users must acknowledge this possibility by unwrapping the returned `Result` before being able to use the created `UtcTime`.

A wide variety of time scales may be encountered in the context of precise timekeeping.
`finetime` provides implementations for the most prevalent time scales: UTC, TAI, terrestrial time (TT), GPS time (GPST), and most other GNSS time scales.
Unix time is explicitly not included, as it is not a continuous time scale: the difference between two Unix times does not reflect the physically elapsed time, because leap seconds are not accounted for.
Where possible, times can be converted between time scales using the `into_time_scale()` function.
```rust
use finetime::{GalileoTime, GpsTime, TaiTime, UtcTime, Month};
let epoch_utc = UtcTime::from_historic_datetime(2025, Month::August, 3, 20, 25, 42).unwrap();
let epoch_tai = TaiTime::from_historic_datetime(2025, Month::August, 3, 20, 26, 19).unwrap();
let epoch_gps = GpsTime::from_historic_datetime(2025, Month::August, 3, 20, 26, 0).unwrap();
let epoch_galileo = GalileoTime::from_historic_datetime(2025, Month::August, 3, 20, 26, 0).unwrap();
assert_eq!(epoch_utc.into_time_scale(), epoch_tai);
assert_eq!(epoch_utc.into_time_scale(), epoch_gps);
assert_eq!(epoch_utc.into_time_scale(), epoch_galileo);
```
If a desired time scale is not present, users may provide their own by implementing the `TimeScale` trait.

There is also support for subsecond datetime values, and conversion into more fine-grained `TimePoint` types to support higher-fidelity time types.
```rust
use finetime::{UtcTime, TtTime, Month, MilliSeconds};
let epoch_utc = UtcTime::from_historic_datetime(2025, Month::August, 3, 20, 25, 42).unwrap();
let epoch_tt = TtTime::from_fine_historic_datetime(2025, Month::August, 3, 20, 26, 51, MilliSeconds::new(184i64)).unwrap();
assert_eq!(epoch_utc.into_unit().into_time_scale(), epoch_tt);
```
These conversions must always be performed explicitly via the `into_unit()` method: this ensures that units are not accidentally mixed. If this is not done, `finetime` simply refuses to compile unit-ambiguous expressions.
Exact integer arithmetic is supported to ensure the absence of round-off errors when converting between units.

## Expressing durations
Within `finetime`, the difference between two `TimePoint`s is expressed as a `Duration`:
```rust
use finetime::{UtcTime, Month, Seconds};
let epoch1 = UtcTime::from_historic_datetime(2020, Month::September, 30, 23, 59, 58).unwrap();
let epoch2 = UtcTime::from_historic_datetime(2020, Month::October, 1, 0, 2, 3).unwrap();
let duration = epoch2 - epoch1;
assert_eq!(duration, Seconds::new(125));
```

Leap second boundaries are handled seamlessly:
```rust
use finetime::{UtcTime, Month, Seconds};
let epoch1 = UtcTime::from_historic_datetime(2016, Month::December, 31, 23, 59, 59).unwrap();
let epoch2 = UtcTime::from_historic_datetime(2016, Month::December, 31, 23, 59, 60).unwrap();
let epoch3 = UtcTime::from_historic_datetime(2017, Month::January, 1, 0, 0, 0).unwrap();
assert_eq!(epoch2 - epoch1, Seconds::new(1));
assert_eq!(epoch3 - epoch2, Seconds::new(1));
assert_eq!(epoch3 - epoch1, Seconds::new(2));
```
The same goes when a time scale does not apply leap seconds:
```rust
use finetime::{TaiTime, Month, Seconds, Second};
let epoch1 = TaiTime::from_historic_datetime(2016, Month::December, 31, 23, 59, 59).unwrap();
let _ = TaiTime::<i64, Second>::from_historic_datetime(2016, Month::December, 31, 23, 59, 60).unwrap_err();
let epoch3 = TaiTime::from_historic_datetime(2017, Month::January, 1, 0, 0, 0).unwrap();
assert_eq!(epoch3 - epoch1, Seconds::new(1i64));
```

As with `TimePoint`s, unit compatibility is checked at compile time, with conversions permit using the `into_unit()` method:
```rust
use finetime::{GpsTime, Month, Hours, MilliSeconds, Second};
let epoch1 = GpsTime::from_historic_datetime(2024, Month::August, 13, 19, 30, 0).unwrap();
let epoch2 = epoch1 + Hours::new(2).into_unit();
let epoch3 = epoch1.into_unit() + MilliSeconds::new(1i64);
assert_eq!(epoch2, GpsTime::from_historic_datetime(2024, Month::August, 13, 21, 30, 0).unwrap());
assert_eq!(epoch3, GpsTime::from_fine_historic_datetime(2024, Month::August, 13, 19, 30, 0, MilliSeconds::new(1)).unwrap());
```

## Casting between representations
Both `TimePoint` and `Duration` are generic over the underlying representation used to represent time spans.
By default, a good choice is `i64` (or `i128` if `i64` overflows), since the resulting time types will not suffer from round-off error.
Sometimes it is desirable to convert to another representation, for whatever reason. In such cases, `cast()` and `try_cast()` may be used, depending on whether the underlying conversion is fallible or not:
```rust
use finetime::{Seconds, MilliSeconds};
let duration_i64 = Seconds::new(3i64);
let duration_float1: Seconds<f64> = duration_i64.try_cast().unwrap();
let duration_i32 = Seconds::new(3i32);
let duration_float2: Seconds<f64> = duration_i32.cast();
assert_eq!(duration_float1, duration_float2);
```

Using `count()`, the raw underlying representation of a `Duration` may be retrieved:
```rust
use finetime::{MilliSeconds};
let duration = MilliSeconds::new(32_184);
let count = duration.count();
assert_eq!(count, 32_184);
```
This representation is nothing more than the number of time units contained in the `Duration`.

## Comparison with `hifitime`, `chrono`, `time`, and `jiff`.
There are a multitude of high-quality Rust timekeeping crates out there already.
In particular, `chrono`, `time`, `jiff`, and `hifitime` will already cover most people's use cases.
Most users will be interested in `jiff`, `chrono` and `time`, which are highly suitable for day-to-day timekeeping.
They handle civilian time zones (which `finetime` does not) and integrate much better with the operating system: however, they do not permit high-accuracy timestamps in frequently-used scientific and engineering time scales, like GPS, TAI, and TT. This makes them unsuitable for astrodynamics, physics, and engineering.
For users that are not interested in such niche applications and time scales, any of `jiff`, `chrono`, and `time` will certainly handle your needs: `finetime` might as well, but is certainly not the only option.

On the other hand, `hifitime` does handle such specialist time scales (although, in turn, it does not cover civilian time scales beyond UTC).
Additionally, `hifitime` supports nanosecond precision and is validated against the timekeeping part of the SPICE astrodynamics library.
Yet, `hifitime`'s `Epoch` type is limited to nanosecond precision and uses a segmented time type that dynamically stores the underlying time standard used.
This introduces quite some complexity in what could be a simple tick counting type.
This complexity definitely does not affect its correctness at all: `hifitime` is well-validated.
However, it does affect the efficiency of the time representation used: `Epoch` always consists of at least an 80-bit `Duration` and an at minimum 8-bit `TimeScale`.
Additionally, this means that the `Epoch` type cannot easily be extended to admit subnanosecond accuracy: in some GNSS applications, such accuracy is becoming necessary.

`finetime` is meant to address these concerns: it efficiently stores the underlying time stamp as a simple tick count. This means that all arithmetic on time stamps reduces to simple arithmetic directly on the underlying tick count: as efficient as it would be when written by hand.
Additionally, the `finetime` `TimePoint` type is generic over the time scale used, the underlying tick count representation, and the units in which it is expressed.
This makes it possible to natively use multiple time stamp types of differing time scale, bitwidth, and precision: all encoded safely and statically, to prevent mix-ups.
Conversion routines are written to support convenient and zero-overhead casting to other `TimePoint` times, where they are compatible.
Consequently, `finetime` is suitable for subnanosecond applications as well as for scenarios where a wide time range must be represented, or at low storage overhead; even within the same program.