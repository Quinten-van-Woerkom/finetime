//! Representation of specific calendrical types, used to represent individual dates according to a
//! variety of historical calendars.

mod historic;
use core::ops::Add;

pub use historic::*;
mod gregorian;
pub use gregorian::*;
mod julian;
pub use julian::*;
mod julian_day;
pub use julian_day::*;
mod modified_julian_date;
pub use modified_julian_date::*;
use thiserror::Error;

use crate::Days;

/// Generic representation of date. Identifies an exact individual date within the calendar, in
/// terms of days before (negative) or after (positive) 1970-01-01. This makes it useful as
/// universal type that can be converted to and from other calendrical types.
///
/// Note that this type is not associated with a time zone: rather, it represents the local time in
/// some implicit time zone.
///
/// It is explicitly not possible to subtract one `Date` from another to obtain a duration. This
/// choice is made to prevent errors due to leap seconds, which cannot be incorporated in a
/// purely calendrical type. Rather, a date must be mapped towards a proper time scale first,
/// before such arithmetic is possible. It is possible to add full days to a `Date`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Constructor)]
pub struct Date<Representation> {
    time_since_epoch: Days<Representation>,
}

impl<Representation> Date<Representation> {
    /// Creates a date from the given number of days since 1970-01-01.
    pub const fn from_time_since_epoch(time_since_epoch: Days<Representation>) -> Self {
        Self { time_since_epoch }
    }

    /// The number of days since the epoch of this representation - midnight 1970.
    pub const fn time_since_epoch(&self) -> Days<Representation>
    where
        Representation: Copy,
    {
        self.time_since_epoch
    }

    /// Casts this date into an equivalent one with another underlying representation. Only
    /// supports lossless conversions.
    pub fn cast<Target>(self) -> Date<Target>
    where
        Representation: Into<Target>,
    {
        Date {
            time_since_epoch: self.time_since_epoch.cast(),
        }
    }

    /// Casts this date into an equivalent one with another underlying representation. Only
    /// supports lossless conversions: if the result would lose information, returns `None`.
    pub fn try_cast<Target>(
        self,
    ) -> Result<Date<Target>, <Representation as TryInto<Target>>::Error>
    where
        Representation: TryInto<Target>,
    {
        Ok(Date {
            time_since_epoch: self.time_since_epoch.try_cast()?,
        })
    }
}

impl Date<i32> {
    /// Creates a `Date` based on a year-month-day date in the historic calendar.
    pub const fn from_historic_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidHistoricDate> {
        match HistoricDate::new(year, month, day) {
            Ok(historic_date) => Ok(historic_date.to_date()),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Gregorian calendar.
    pub const fn from_gregorian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidGregorianDate> {
        match GregorianDate::new(year, month, day) {
            Ok(gregorian_date) => Ok(gregorian_date.to_date()),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Julian calendar.
    pub const fn from_julian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidJulianDate> {
        match JulianDate::new(year, month, day) {
            Ok(julian_date) => Ok(julian_date.to_date()),
            Err(error) => Err(error),
        }
    }
}

impl<Representation> Add<Days<Representation>> for Date<Representation>
where
    Representation: Add,
{
    type Output = Date<<Representation as Add>::Output>;

    fn add(self, rhs: Days<Representation>) -> Self::Output {
        Date {
            time_since_epoch: self.time_since_epoch + rhs,
        }
    }
}

#[cfg(kani)]
impl<Representation> kani::Arbitrary for Date<Representation>
where
    Representation: kani::Arbitrary,
{
    fn any() -> Self {
        Self::from_time_since_epoch(Days::any())
    }
}

/// Representation of a month in a Roman calendar.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
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
    pub const fn from_raw(month: u8) -> Result<Self, InvalidMonthNumber> {
        let month = match month {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => return Err(InvalidMonthNumber { month }),
        };
        Ok(month)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid month number {month}")]
pub struct InvalidMonthNumber {
    month: u8,
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

/// Verifies that the epoch of `Date` is found at 1970-01-01 (historic calendar).
#[test]
fn epoch_at_1970_01_01() {
    let epoch = Date::from_historic_date(1970, Month::January, 1).unwrap();
    assert_eq!(epoch.time_since_epoch.count(), 0);

    let historic_date = HistoricDate::new(1970, Month::January, 1).unwrap();
    let historic_date2 = HistoricDate::from_date(epoch);
    assert_eq!(historic_date, historic_date2);
}
