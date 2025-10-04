//! Implementation of the `Month` data type.

use crate::errors::InvalidMonthNumber;

/// Representation of a month in a Roman calendar.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::TryFrom,
)]
#[try_from(repr)]
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

impl Month {
    pub const fn try_from(month: u8) -> Result<Self, InvalidMonthNumber> {
        let month = match month {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => return Err(InvalidMonthNumber { month }),
        };
        Ok(month)
    }
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
