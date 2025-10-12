//! Implementation of Coordinated Universal Time (UTC).

use core::ops::{Add, Sub};

use crate::{
    ConvertUnit, Date, FromDateTime, Hours, LeapSecondProvider, Minutes, Month, Second, Seconds,
    StaticLeapSecondProvider, TerrestrialTime, TimePoint, TryFromExact, Years,
    errors::{InvalidTimeOfDay, InvalidUtcDateTime},
    time_scale::TimeScale,
    units::{SecondsPerDay, SecondsPerHour, SecondsPerMinute, SecondsPerYear},
};

pub type UtcTime<Representation = i64, Period = Second> = TimePoint<Utc, Representation, Period>;

/// Time scale representing Coordinated Universal Time (UTC). This scale is adjusted using leap
/// seconds to closely match the rotation of the Earth. This makes it useful as civil time scale,
/// but also means that external, discontinuous synchronization is required.
///
/// The synchronization based on leap seconds is implemented to occur at the date-time boundary.
/// This means that it is only done when a UTC time point is created based on a date-time pair,
/// after which it is converted into a time-since-epoch representation. This makes arithmetic over
/// UTC time points much more efficient and entirely correct over all possible leap second
/// boundaries.
///
/// This choice does also mean that introduction of new leap seconds does not "shift" any UTC time
/// stamps that were created to be after the point of introduction of this leap second. Generally,
/// this is desired behaviour, but in human communication it might not be. In such cases, users are
/// better off storing their UTC timestamps as date-time pairs and only converting them into
/// `UtcTime` at the point of use.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Utc;

impl TimeScale for Utc {
    const NAME: &'static str = "Coordinated Universal Time";

    const ABBREVIATION: &'static str = "UTC";

    /// This epoch is the exact date at which the modern definition of UTC started. This makes it
    /// useful, because users may choose to permit "proleptic" UTC dates before 1972 by using a
    /// signed representation, but may also choose to forbid it by using unsigned arithmetic, which
    /// leads to easy-to-detect underflows whenever an ambiguous pre-1972 UTC date-time is created.
    const EPOCH: Date<i32> = match Date::from_gregorian_date(1972, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl TerrestrialTime for Utc {
    type Representation = u8;

    type Period = SecondsPerYear;

    /// Perhaps confusingly, we define UTC as coinciding with TAI. This is entirely possible
    /// because we handle leap seconds at the date-time boundary: after converting UTC into its
    /// time-since-epoch variation, there are no leap seconds to speak of anymore.
    const TAI_OFFSET: Years<u8> = Years::new(0);
}

impl<Representation> FromDateTime for UtcTime<Representation, Second>
where
    Representation: ConvertUnit<SecondsPerMinute, Second>
        + ConvertUnit<SecondsPerHour, Second>
        + ConvertUnit<SecondsPerDay, Second>
        + Add<Representation, Output = Representation>
        + Sub<Representation, Output = Representation>
        + TryFromExact<i32>
        + TryFromExact<u8>,
{
    type Error = InvalidUtcDateTime;

    fn from_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, Self::Error> {
        if hour > 23 || minute > 59 || second > 60 {
            return Err(InvalidUtcDateTime::InvalidTimeOfDay(InvalidTimeOfDay {
                hour,
                minute,
                second,
            }));
        }

        let (is_leap_second, total_leap_seconds) =
            StaticLeapSecondProvider {}.leap_seconds_on_date(date);
        if second == 60 && !is_leap_second {
            return Err(InvalidUtcDateTime::NonLeapSecondDateTime {
                date,
                hour,
                minute,
                second,
            });
        }

        let days_since_scale_epoch = {
            let days_since_1970 = date.time_since_epoch();
            let epoch_days_since_1970 = Utc::EPOCH.time_since_epoch();

            // First we try to compute the difference by subtracting first and then converting into
            // the target representation.
            let difference = (days_since_1970 - epoch_days_since_1970).try_cast::<Representation>();
            if let Ok(difference) = difference {
                difference
            } else {
                // If that fails, we try first casting into the target representation and then
                // subtracting. If that also fails, we just error.
                days_since_1970
                    .try_cast::<Representation>()
                    .unwrap_or_else(|_| panic!())
                    - epoch_days_since_1970
                        .try_cast::<Representation>()
                        .unwrap_or_else(|_| panic!())
            }
        };

        let hours = Hours::new(hour)
            .try_cast::<Representation>()
            .unwrap_or_else(|_| panic!());
        let minutes = Minutes::new(minute)
            .try_cast::<Representation>()
            .unwrap_or_else(|_| panic!());
        let seconds = Seconds::new(second)
            .try_cast::<Representation>()
            .unwrap_or_else(|_| panic!());
        let time_since_epoch = days_since_scale_epoch.into_unit()
            + hours.into_unit()
            + minutes.into_unit()
            + seconds
            + total_leap_seconds
                .try_cast::<Representation>()
                .unwrap_or_else(|_| panic!());
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }
}

/// Tests the creation of UTC time points from calendar dates for some known values. We explicitly
/// try out times near leap second insertions to see if those are handled properly, including:
/// - Durations should be handled correctly before, during, and after a leap second.
/// - If a leap second format (61 seconds) datetime is given for a non-leap second datetime, this
///   shall be caught and indicated.
#[test]
fn calendar_dates_near_insertion() {
    use crate::Month::*;
    // Leap second insertion of June 2015.
    let date = Date::from_historic_date(2015, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::from_historic_date(2015, July, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Leap second insertion of December 2016.
    let date = Date::from_historic_date(2016, December, 31).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::from_historic_date(2017, January, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Non-leap second date: June 2016
    let date = Date::from_historic_date(2016, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::<i64, Second>::from_datetime(date, 23, 59, 60);
    assert_eq!(
        leap_second,
        Err(InvalidUtcDateTime::NonLeapSecondDateTime {
            date,
            hour: 23,
            minute: 59,
            second: 60
        })
    );
}

#[test]
fn trivial_times() {
    let epoch = UtcTime::from_historic_datetime(1972, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(epoch.time_since_epoch(), Seconds::new(10));
    let epoch = UtcTime::from_historic_datetime(1971, Month::December, 31, 23, 59, 60).unwrap();
    assert_eq!(epoch.time_since_epoch(), Seconds::new(9));
}
