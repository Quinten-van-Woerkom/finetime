//! Implementation of a time scale that represents that used by Unix time, i.e., the system clock.

use crate::{
    calendar::{
        Date, Datelike,
        Month::{self, *},
    },
    duration::{Hours, Minutes, Seconds, units::LiteralRatio},
    time_point::TimePoint,
    time_scale::{
        TimeScale,
        local::LocalDays,
        tai::{Tai, TaiTime},
    },
};

/// `UnixTime` is a `TimePoint` that uses the `Unix` time scale.
pub type UnixTime<Representation, Period = LiteralRatio<1>> =
    TimePoint<Unix, Representation, Period>;

impl UnixTime<i64> {
    /// Creates a Unix time point from a given historic calendar date and time stamp. Note that
    /// Unix time points are expressed in UTC, but do not include leap seconds.
    pub fn from_datetime(
        date: impl Datelike,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<UnixTime<i64>, UnixTimeError> {
        Self::from_local_datetime(date.into(), hour, minute, second)
    }

    pub fn from_local_datetime(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<UnixTime<i64>, UnixTimeError> {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(UnixTimeError::TimeDoesNotExist {
                hour,
                minute,
                second,
            });
        }

        // Afterwards, we convert the date to its MJD equivalent. We do the same for the Unix
        // epoch, but then at compile time already. Note that both dates are MJD, expressed in Unix
        // time.
        let date_mjd = date;
        let unix_epoch = Unix::epoch();
        let days = date_mjd - unix_epoch;
        let hours = Hours::new(hour as i64);
        let minutes = Minutes::new(minute as i64);
        let seconds = Seconds::new(second as i64);
        Ok(TimePoint::from_time_since_epoch(
            days.convert() + hours.convert() + minutes.convert() + seconds,
        ))
    }
}

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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Unix;

impl Unix {
    /// Returns the Unix epoch as a `LocalDays`. Note that it is still expressed in UTC, so may not
    /// be compared directly with other time scale epochs like that of TAI.
    pub const fn epoch() -> LocalDays<i64> {
        match Date::new(1970, Month::January, 1) {
            Ok(date) => LocalDays::from_date(date),
            Err(_) => panic!("Internal error: Unix epoch was found to be an invalid date."),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnixTimeError {
    /// Returned when the requested time-of-day is not a valid timestamp.
    TimeDoesNotExist { hour: u8, minute: u8, second: u8 },
}

impl TimeScale for Unix {
    /// The Unix reference epoch is 1 January 1970 midnight UTC.
    fn reference_epoch() -> crate::time_point::TimePoint<Tai, i64, crate::duration::units::Milli> {
        let date = Date::new(1970, January, 1).unwrap();
        TaiTime::from_datetime(date, 0, 0, 10).unwrap().convert()
    }
}
