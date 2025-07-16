//! Implementation of international atomic time (TAI).

use num::{NumCast, Zero};

use crate::{
    calendar::{Date, Month},
    duration::{
        MilliSeconds,
        units::{LiteralRatio, Milli},
    },
    time_point::TimePoint,
    time_scale::{TimeScale, local::LocalDays},
};

/// `TaiTime` is a specialization of `TimePoint` that uses the TAI time scale.
pub type TaiTime<Representation, Period = LiteralRatio<1>> = TimePoint<Tai, Representation, Period>;

/// Time scale representing the international atomic time standard (TAI). TAI has no leap seconds
/// and increases monotonically at a constant interval, making it useful as fundamental time scale
/// to build the rest of this library on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tai;

impl Tai {
    /// Returns the TAI epoch as a `LocalDays`. Note that this `LocalDays` itself is still
    /// expressed in TAI.
    pub const fn epoch() -> LocalDays<i64> {
        match Date::new(1958, Month::January, 1) {
            Ok(date) => date.to_local_days(),
            Err(_) => panic!("Internal error: TAI epoch was found to be an invalid date."),
        }
    }
}

/// Error that may be returned when creating a TAI time point from a calendar representation. Note
/// that this calendar representation does not allow leap seconds, as opposed to the equivalent
/// calendar representation of UTC time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaiError {
    /// Returned when the given time-of-day is not a valid timestamp.
    TimeDoesNotExist { hour: u8, minute: u8, second: u8 },
}

impl TimeScale for Tai {
    /// Since TAI is used as central time scale, its own reference epoch is at time point 0.
    fn reference_epoch() -> TimePoint<Tai, i64, Milli> {
        TimePoint::from_time_since_epoch(MilliSeconds::zero())
    }

    fn epoch<T>() -> LocalDays<T>
    where
        T: NumCast,
    {
        Date::new(1958, Month::January, 1)
            .unwrap()
            .to_local_days()
            .cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TAI time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TaiTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a TAI time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TaiTime::from_datetime(date, hour, minute, second);
    }
}

/// Verifies this implementation by computing the `TaiTime` for some known time stamps. To compute
/// these known time stamps, we use the fact that TAI, just like Unix time, has days that are
/// always exactly 86,400 seconds long. Hence, the differences are caused by only an offset in
/// epoch, which is the difference between 1958 and 1970: 378691200 seconds.
#[test]
fn known_timestamps() {
    use crate::duration::Seconds;
    assert_eq!(
        TaiTime::from_datetime(Date::new(1958, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(0)
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1958, Month::January, 2).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1960, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2 * 365 * 24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1961, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new((3 * 365 + 1) * 24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1970, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1976, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(189302400 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2025, Month::July, 16).unwrap(), 16, 23, 24)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752683004 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2034, Month::December, 26).unwrap(), 8, 2, 37)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2050732957 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2760, Month::April, 1).unwrap(), 21, 59, 58)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24937883998 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1643, Month::January, 4).unwrap(), 1, 1, 33)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(-10318834707 + 378691200),
    );
}
