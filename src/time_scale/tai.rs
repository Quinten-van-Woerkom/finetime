//! Implementation of international atomic time (TAI).

use num::Zero;

use crate::{
    calendar::{Date, GregorianDate, ModifiedJulianDay, Month},
    duration::{
        Hours, MilliSeconds, Minutes, Seconds,
        units::{LiteralRatio, Milli},
    },
    time_point::TimePoint,
    time_scale::TimeScale,
};

/// `TaiTime` is a specialization of `TimePoint` that uses the TAI time scale.
pub type TaiTime<Representation, Period = LiteralRatio<1>> = TimePoint<Tai, Representation, Period>;

impl TaiTime<i64> {
    /// Creates a TAI time point from a given historic calendar date and time stamp. Note that leap
    /// seconds are not included, as opposed to the equivalent calendar date representation of UTC
    /// time. This has the advantages that leap seconds need not be accounted for when converting
    /// to the time since epoch, only leap days.
    pub fn from_datetime(
        date: Date,
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
        let date_mjd = ModifiedJulianDay::from_date(date);
        let tai_epoch = ModifiedJulianDay::from_date(Tai::epoch_as_date());
        let days = date_mjd - tai_epoch;
        let hours = Hours::new(hour as i64);
        let minutes = Minutes::new(minute as i64);
        let seconds = Seconds::new(second as i64);
        Ok(TimePoint::from_time_since_epoch(
            days.convert() + hours.convert() + minutes.convert() + seconds,
        ))
    }

    /// Creates a TAI time point from a given Gregorian calendar date. Note that leap seconds are
    /// not included, as opposed to the equivalent Gregorian calendar representation of UTC time.
    /// This does have the advantage that leap seconds need not be accounted for when converting to
    /// the time since epoch, only leap days.
    ///
    /// This function will return the same value as `from_datetime` for all modern dates. Only for
    /// values before the Gregorian calendar reform (15 October 1582) will a difference occur. The
    /// `from_gregorian` function is in such cases less historically accurate, but it is what
    /// `hifitime` and `chrono` do - hence it is included here for completeness, too. In practice,
    /// it is recommended to stick to `from_date`, since that is in line with the choices made by
    /// NAIF SPICE and IAU SOFA.
    pub fn from_gregorian_datetime(
        date: GregorianDate,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Tai, i64>, TaiError> {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(TaiError::TimeDoesNotExist {
                hour,
                minute,
                second,
            });
        }

        // Afterwards, we convert the Gregorian date to its MJD equivalent. We do the same for the
        // TAI epoch, but then at compile time already.
        let date_mjd = ModifiedJulianDay::from_gregorian_date(date);
        let tai_epoch = ModifiedJulianDay::from_date(Tai::epoch_as_date());
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
    /// Returns the TAI epoch as a date. Note that this date itself is still expressed in TAI.
    pub const fn epoch_as_date() -> Date {
        match Date::new(1958, Month::January, 1) {
            Ok(date) => date,
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
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TaiTime::from_gregorian_datetime(date, hour, minute, second);
    }
}
