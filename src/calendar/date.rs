//! Implementation of the "generic" date representation, which is largely agnostic of any specific
//! calendar representation: including phenomena such as months, weeks, years, and leap years.
//! Rather, it is a simple day count since the Unix epoch.

use core::ops::Add;

use crate::{
    Days, GregorianDate, HistoricDate, JulianDate, Month, WeekDay,
    errors::{InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate},
};

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

    /// Returns the day-of-the-week of this date.
    pub const fn week_day(&self) -> WeekDay {
        let z = self.time_since_epoch().count();
        let day = if z >= -4 {
            (z + 4) % 7
        } else {
            (z + 5) % 7 + 6
        };
        match WeekDay::try_from(day as u8) {
            Ok(week_day) => week_day,
            Err(_) => unreachable!(),
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

/// Verifies that the epoch of `Date` is found at 1970-01-01 (historic calendar).
#[test]
fn epoch_at_1970_01_01() {
    let epoch = Date::from_historic_date(1970, Month::January, 1).unwrap();
    assert_eq!(epoch.time_since_epoch(), Days::new(0));

    let historic_date = HistoricDate::new(1970, Month::January, 1).unwrap();
    let historic_date2 = HistoricDate::from_date(epoch);
    assert_eq!(historic_date, historic_date2);
}

/// Tests some known week day values.
#[test]
fn week_days() {
    assert_eq!(
        Date::from_historic_date(1970, Month::January, 1)
            .unwrap()
            .week_day(),
        WeekDay::Thursday
    );

    assert_eq!(
        Date::from_historic_date(1998, Month::December, 17)
            .unwrap()
            .week_day(),
        WeekDay::Thursday
    );
}
