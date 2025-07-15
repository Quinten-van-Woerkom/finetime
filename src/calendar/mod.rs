//! Implementation of all calendar-related functionality.

pub mod gregorian;
pub mod historic;
pub mod modified_julian_day;

pub use gregorian::*;
pub use historic::*;
pub use modified_julian_day::*;

use crate::time_scale::local::LocalDays;

/// Trait describing anything that can be interpreted as a date. In practice, this means anything
/// that can be converted to `LocalDays`, i.e., a day-accurate timestamp that is not yet bound to a
/// timezone but that is related in some manner to actual time.
///
/// The reverse transformation is not required, while it would generally make sense, because it can
/// be non-trivial to implement for certain calendars.
pub trait Datelike: Into<LocalDays<i64>> {}

/// Months as known in the Gregorian and Julian calendars.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

#[cfg(kani)]
impl kani::Arbitrary for Month {
    fn any() -> Self {
        use Month::*;
        let any: u8 = (kani::any::<u8>() % 12u8) + 1u8;
        match any {
            1 => January,
            2 => February,
            3 => March,
            4 => April,
            5 => May,
            6 => June,
            7 => July,
            8 => August,
            9 => September,
            10 => October,
            11 => November,
            12 => December,
            _ => unreachable!(),
        }
    }
}
