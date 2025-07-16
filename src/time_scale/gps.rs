//! Implementation of the Global Positioning System (GPS) time scale.

use crate::{
    calendar::{Date, Month},
    time_scale::{
        TimeScale, TimeScaleConversion,
        tai::{Tai, TaiTime},
    },
};

/// The Global Positioning System (GPS) time scale is broadcast by GPS satellites. It is based on
/// internal atomic clocks that are synchronized with TAI. The signal is defined to be a constant
/// 19 seconds behind TAI.
pub struct Gps;

impl TimeScale for Gps {
    fn reference_epoch()
    -> crate::time_point::TimePoint<super::tai::Tai, i64, crate::duration::units::Milli> {
        TaiTime::from_datetime(Date::new(1980, Month::January, 6).unwrap(), 0, 0, 19)
            .unwrap()
            .convert()
    }

    fn epoch<T>() -> super::local::LocalDays<T>
    where
        T: num::NumCast,
    {
        Date::new(1980, Month::January, 6)
            .unwrap()
            .to_local_days()
            .try_cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl TimeScaleConversion<Tai, Gps> for () {}
impl TimeScaleConversion<Gps, Tai> for () {}
