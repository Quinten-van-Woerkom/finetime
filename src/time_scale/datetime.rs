//! Implementation of the concept of date and time-of-day within a time scale.

use crate::{
    Date, Hours, Minutes, Seconds, TimePoint,
    errors::InvalidTimeOfDay,
    units::{Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute},
};

/// This trait may be implemented for time scales that are able to handle the concept of a
/// date-time pair: they can connect a date and time-of-day to a specific time instant within their
/// internal scale and back.
///
/// Note that this mapping is explicitly only performed at the precision of seconds and with only
/// `i64` as underlying representation. These base implementations are "easy" to implement once,
/// and can then be mapped to other representations and/or smaller units via simple casts. A time
/// span of 292 billion years before and after some epoch may be represented with this combination
/// of unit and representation: this should be fine for all time scales on which calendrical math
/// may be applied - it is about twenty times the age of the universe.
pub trait DateTime {
    type Error: core::error::Error;

    /// Maps a given combination of date and time-of-day to an instant on this time scale. May
    /// return an error if the input does not represent a valid combination of date and
    /// time-of-day.
    fn time_point_from_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, i64, Second>, Self::Error>;

    /// Maps a time point back to the date and time-of-day that it represents. Returns a tuple of
    /// date, hour, minute, and second. This function shall not fail, unless overflow occurs in the
    /// underlying integer arithmetic.
    fn datetime_from_time_point(
        time_point: TimePoint<Self, i64, Second>,
    ) -> (Date<i32>, u8, u8, u8);
}

/// Some date-time scales are continuous: they do not apply leap seconds. In such cases, their
/// implementation of the `DateTime` mapping reduces to a simple add-and-multiply of days, hours,
/// minutes, and seconds with respect to the "arbitrary" measurement epoch in which their resulting
/// time points are measured.
pub trait ContinuousDateTime {
    /// Determines the epoch used to convert date-time of this time scale into the equivalent
    /// time-since-epoch `TimePoint` representation. For simplicity, epochs must fall on date
    /// boundaries.
    ///
    /// Note that this epoch does not bear any "real" significance: it is merely a representational
    /// choice. In principle, it may even be distinct from the "actual" epoch, if any is defined,
    /// for a time scale. For GPS, for example, the epoch is well-defined as 1980-01-06T00:00:00
    /// UTC, but it would not necessarily be wrong to use a different date here. In practice, of
    /// course, it is more convenient to choose the actual epoch where one is defined.
    const EPOCH: Date<i32>;
}

impl<Scale> DateTime for Scale
where
    Scale: ContinuousDateTime,
{
    type Error = InvalidTimeOfDay;

    /// When a continuous date-time mapping exists (without leap seconds), the `TimePoint`
    /// corresponding with a given date-time may be computed by adding the days, hours, minutes,
    /// and seconds since some epoch.
    fn time_point_from_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, i64, Second>, Self::Error> {
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }
        let days_since_scale_epoch =
            date.time_since_epoch().cast::<i64>() - Self::EPOCH.time_since_epoch().cast::<i64>();
        let hours = Hours::new(hour).cast::<i64>();
        let minutes = Minutes::new(minute).cast::<i64>();
        let seconds = Seconds::new(second).cast::<i64>();
        let time_since_epoch = days_since_scale_epoch.into_unit()
            + hours.into_unit()
            + minutes.into_unit()
            + seconds.into_unit();
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }

    /// When a continuous date-time mapping exists (without leap seconds), the date-time
    /// corresponding with some `TimePoint` may be computed by factoring out the days, hours,
    /// minutes, and seconds since some epoch.
    fn datetime_from_time_point(
        time_point: TimePoint<Self, i64, Second>,
    ) -> (Date<i32>, u8, u8, u8) {
        // Step-by-step factoring of the time since epoch into days, hours, minutes, and seconds.
        let seconds_since_scale_epoch = time_point.time_since_epoch();
        let (days_since_scale_epoch, seconds_in_day) =
            seconds_since_scale_epoch.factor_out::<SecondsPerDay>();
        let (hour, seconds_in_hour) = seconds_in_day.factor_out::<SecondsPerHour>();
        let (minute, second) = seconds_in_hour.factor_out::<SecondsPerMinute>();
        let days_since_universal_epoch =
            Self::EPOCH.time_since_epoch().cast::<i64>() + days_since_scale_epoch;
        let date = Date::from_time_since_epoch(days_since_universal_epoch);

        // We must narrow-cast all results, but only the cast of `date` may fail. The rest will
        // always succeed by construction: hour < 24, minute < 60, second < 60, so all fit in `u8`.
        (
            date.try_cast()
                .expect("Call of `date_time_from_time_point` results in date outside of representable range of `i32`"),
            hour.count() as u8,
            minute.count() as u8,
            second.count() as u8,
        )
    }
}
