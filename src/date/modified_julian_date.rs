//! The Modified Julian Day (MJD) is nothing more than the number of days since 1858 November 17
//! at 0h UT. Effectively, this makes it a constant offset from the Julian Day (JD); however, the
//! MJD is useful because it is not fractional for time points at midnight.

use core::ops::{Add, Sub};

use crate::{
    Convert, Date, Duration, InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate, Month,
    UnitRatio, duration::Days, units::SecondsPerDay,
};

/// The Modified Julian Day (MJD) representation of any given date.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifiedJulianDate<Representation, Period = SecondsPerDay> {
    day: Duration<Representation, Period>,
}

/// The Julian date of the Unix epoch is useful as constant in some calculations.
pub const MODIFIED_JULIAN_DATE_UNIX_EPOCH: Days<i32> = Days::new(40587);

impl<Representation, Period> ModifiedJulianDate<Representation, Period>
where
    Period: UnitRatio,
{
    /// Constructs a new MJD directly from some duration since the MJD epoch, November 17 1858.
    pub const fn from_time_since_epoch(day: Duration<Representation, Period>) -> Self {
        Self { day }
    }
}

impl<Representation, Period> ModifiedJulianDate<Representation, Period> {
    /// Constructs a Julian date from some given calendar date.
    pub fn from_date(date: Date<Representation>) -> Self
    where
        Representation: Copy
            + From<i32>
            + Add<Representation, Output = Representation>
            + Convert<SecondsPerDay, Period>,
    {
        Self {
            day: date.time_since_epoch().into_unit()
                + MODIFIED_JULIAN_DATE_UNIX_EPOCH.cast().into_unit(),
        }
    }

    pub fn to_date(&self) -> Date<Representation>
    where
        Representation: Copy
            + From<i32>
            + Sub<Representation, Output = Representation>
            + Convert<Period, SecondsPerDay>,
    {
        Date::from_time_since_epoch(self.day.into_unit() - MODIFIED_JULIAN_DATE_UNIX_EPOCH.cast())
    }
}

impl ModifiedJulianDate<i64> {
    /// Creates a `Date` based on a year-month-day date in the historic calendar.
    pub fn from_historic_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidHistoricDate> {
        match Date::from_historic_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Gregorian calendar.
    pub fn from_gregorian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidGregorianDate> {
        match Date::from_gregorian_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Julian calendar.
    pub fn from_julian_date(year: i32, month: Month, day: u8) -> Result<Self, InvalidJulianDate> {
        match Date::from_julian_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }
}

impl<Representation, Period> From<Date<Representation>>
    for ModifiedJulianDate<Representation, Period>
where
    Representation: Copy
        + From<i32>
        + Add<Representation, Output = Representation>
        + Convert<SecondsPerDay, Period>,
{
    fn from(value: Date<Representation>) -> Self {
        Self::from_date(value)
    }
}

impl<Representation, Period> From<ModifiedJulianDate<Representation, Period>>
    for Date<Representation>
where
    Representation: Copy
        + From<i32>
        + Sub<Representation, Output = Representation>
        + Convert<Period, SecondsPerDay>,
{
    fn from(value: ModifiedJulianDate<Representation, Period>) -> Self {
        value.to_date()
    }
}

/// Compares some computed MJD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::Month::*;
    assert_eq!(
        ModifiedJulianDate::from_historic_date(2000, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(51544))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1999, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(51179))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1987, January, 27).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(46822))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1987, June, 19).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(46965))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1988, January, 27).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(47187))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1988, June, 19).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(47331))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1900, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(15020))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1600, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-94553))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(1600, December, 31).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-94188))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(837, April, 10).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-373129))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-123, December, 31).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-723504))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-122, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-723503))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-1000, July, 12).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-1044000))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-1000, February, 29).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-1044134))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-1001, August, 17).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-1044330))
    );
    assert_eq!(
        ModifiedJulianDate::from_historic_date(-4712, January, 1).unwrap(),
        ModifiedJulianDate::from_time_since_epoch(Days::new(-2400001))
    );
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a MJD based on a historic date never panics.
    #[kani::proof]
    fn from_historic_date_never_panics() {
        use crate::HistoricDate;
        let date: HistoricDate = kani::any();
        let _ = ModifiedJulianDate::<i64>::from_date(date.into());
    }

    /// Verifies that construction of a MJD based on a Gregorian date never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::GregorianDate;
        let date: GregorianDate = kani::any();
        let _ = ModifiedJulianDate::<i64>::from_date(date.into());
    }
}
