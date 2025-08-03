//! Implementation of a time scale that represents that used by Unix time, i.e., the system clock.

use crate::{
    FromTimeScale, LocalTime,
    arithmetic::{Second, TimeRepresentation, Unit},
    duration::Duration,
    time_point::TimePoint,
    time_scale::{TimeScale, local::LocalDays},
};

/// `UnixTime` is a `TimePoint` that uses the `Unix` time scale.
pub type UnixTime<Representation, Period = Second> = TimePoint<Unix, Representation, Period>;

/// The Unix time scale is the scale used throughout most operating systems nowadays. It is also
/// the default used in `libc`, for example. It counts seconds since the Unix epoch (1970-01-01 UTC),
/// but excludes leap seconds. In other words, Unix time stops for a second during a leap second.
/// Outside of leap seconds, this means that the same exact time will be shown as during UTC, but
/// durations that span across leap seconds are off by a second. Also, time stamps stored during
/// leap seconds will be ambiguous.
///
/// Notably, the Unix epoch is placed before the official definition of UTC as we know it now (the
/// first leap seconds are defined only at 1972-01-01, starting with a jump of 10 seconds). The way
/// to handle this differs per implementation:
/// - NAIF SPICE uses 9 leap seconds for all times before 1972-01-01.
/// - SOFA's `iauDat` returns non-integer leap seconds to reflect the actual evolution of UTC over
///   these years, where frequency steering was used.
/// - `hifitime` uses 0 leap seconds, starting with a jump to 10 leap seconds at 1972-01-01.
///
/// After 1972-01-01 all these implementations align, so the actual choice is of little
/// consequence. This implementation follows `hifitime`, because it is the easiest to implement.
/// Practically, it permits the Unix timescale to be implemented as a constant offset from TAI with
/// an epoch at exactly midnight 1970-01-01.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unix;

impl Unix {
    /// Returns the Unix epoch as a `LocalDays`. Note that it is still expressed in UTC, so may not
    /// be compared directly with other time scale epochs like that of TAI.
    pub const fn epoch() -> LocalDays<i64> {
        LocalDays::from_time_since_epoch(Duration::new(0))
    }
}

impl TimeScale for Unix {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    /// Because the Unix epoch coincides with the `LocalDays` epoch, it can be constructed simply
    /// as a zero value.
    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        LocalDays::from_time_since_epoch(Duration::new(0))
            .into_unit()
            .try_cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl FromTimeScale<Unix> for Unix {
    fn from_time_scale<Representation, Period>(
        from: TimePoint<Unix, Representation, Period>,
    ) -> TimePoint<Self, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation,
    {
        from
    }
}

#[cfg(feature = "std")]
impl From<std::time::SystemTime> for UnixTime<u128, crate::arithmetic::Nano> {
    fn from(value: std::time::SystemTime) -> Self {
        let nanoseconds_since_epoch = crate::NanoSeconds::new(
            value
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        );
        Self::from_time_since_epoch(nanoseconds_since_epoch)
    }
}

/// Verifies this implementation by computing the `UnixTime` for some known time stamps.
#[test]
fn known_timestamps() {
    use crate::{Date, Month, Seconds};
    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1970, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(0)
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1970, Month::January, 2).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24 * 60 * 60),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1972, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2 * 365 * 24 * 60 * 60),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1973, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new((3 * 365 + 1) * 24 * 60 * 60),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1976, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(189302400),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(2025, Month::July, 16).unwrap(), 16, 23, 24)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752683004),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(2034, Month::December, 26).unwrap(), 8, 2, 37)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2050732957),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(2760, Month::April, 1).unwrap(), 21, 59, 58)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24937883998),
    );

    assert_eq!(
        UnixTime::from_generic_datetime(Date::new(1643, Month::January, 4).unwrap(), 1, 1, 33)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(-10318834707),
    );
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::Month;

    /// Verifies that construction of a Unix time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = UnixTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a Unix time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = UnixTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }
}
