//! Implementation of all calendar-related functionality.

mod gregorian;
mod historic;
mod mjd;

pub use gregorian::*;
pub use historic::*;
pub use mjd::*;

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
