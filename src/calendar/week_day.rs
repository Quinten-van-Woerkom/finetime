//! Implementation of the `WeekDay` type, used to represent days of the week.

use crate::errors::InvalidWeekDayNumber;

/// Indication of a specific day-of-the-week. While explicit values are assigned to each day (to
/// make implementation easier), no ordering is implied.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, derive_more::Display, derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum WeekDay {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

impl WeekDay {
    pub const fn try_from(week_day: u8) -> Result<Self, InvalidWeekDayNumber> {
        let week_day = match week_day {
            0 => Self::Sunday,
            1 => Self::Monday,
            2 => Self::Tuesday,
            3 => Self::Wednesday,
            4 => Self::Thursday,
            5 => Self::Friday,
            6 => Self::Saturday,
            _ => return Err(InvalidWeekDayNumber { week_day }),
        };
        Ok(week_day)
    }
}
