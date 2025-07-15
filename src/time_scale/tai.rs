//! Implementation of international atomic time (TAI).

use num::Zero;

use crate::{
    calendar::{Date, Datelike, Month},
    duration::{
        Hours, MilliSeconds, Minutes, Seconds,
        units::{LiteralRatio, Milli},
    },
    time_point::TimePoint,
    time_scale::{TimeScale, local::LocalDays},
};

/// `TaiTime` is a specialization of `TimePoint` that uses the TAI time scale.
pub type TaiTime<Representation, Period = LiteralRatio<1>> = TimePoint<Tai, Representation, Period>;

impl TaiTime<i64> {
    /// Creates a TAI time point from a given historic calendar date and time stamp. Note that leap
    /// seconds are not included, as opposed to the equivalent calendar date representation of UTC
    /// time. This has the advantages that leap seconds need not be accounted for when converting
    /// to the time since epoch, only leap days.
    pub fn from_datetime(
        date: impl Datelike,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TaiTime<i64>, TaiError> {
        Self::from_local_datetime(date.into(), hour, minute, second)
    }

    pub fn from_local_datetime(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TaiTime<i64>, TaiError> {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(TaiError::TimeDoesNotExist {
                hour,
                minute,
                second,
            });
        }

        // Afterwards, we convert the date to its MJD equivalent. We do the same for the TAI epoch,
        // but then at compile time already. Note that both dates are MJD, expressed in TAI.
        let date_mjd = date;
        let tai_epoch = Tai::epoch();
        let days = date_mjd - tai_epoch;
        let hours = Hours::new(hour as i64);
        let minutes = Minutes::new(minute as i64);
        let seconds = Seconds::new(second as i64);
        Ok(TimePoint::from_time_since_epoch(
            days.convert() + hours.convert() + minutes.convert() + seconds,
        ))
    }
}

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
            Ok(date) => LocalDays::from_date(date),
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
