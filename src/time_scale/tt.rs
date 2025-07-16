//! Implementation of the Terrestrial Time (TT) time scale.

use crate::{
    calendar::{Date, Month},
    duration::MilliSeconds,
    time_scale::{
        TimeScale, TimeScaleConversion,
        tai::{Tai, TaiTime},
    },
};

/// Terrestrial time is the proper time of a clock located on the Earth geoid. It is used in
/// astronomical tables, mostly. Effectively, it is little more than a constant offset from TAI.
pub struct Tt;

impl TimeScale for Tt {
    /// Terrestrial time is exactly (by definition) 32.184 seconds ahead of TAI.
    fn reference_epoch()
    -> crate::time_point::TimePoint<super::tai::Tai, i64, crate::duration::units::Milli> {
        TaiTime::from_datetime(Date::new(1958, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .convert()
            + MilliSeconds::new(32_184)
    }

    /// Terrestrial time does not have an actual epoch associated with it. For practical purposes,
    /// it is useful to choose January 1, 1958, same as TAI.
    fn epoch<T>() -> super::local::LocalDays<T>
    where
        T: num::NumCast,
    {
        Tai::epoch()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl TimeScaleConversion<Tt, Tai> for () {}
impl TimeScaleConversion<Tai, Tt> for () {}
