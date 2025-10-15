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
#[cfg_attr(kani, derive(kani::Arbitrary))]
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
