//! The Modified Julian Day (MJD) is nothing more than the number of days since 1858 November 17
//! at 0h UT. Effectively, this makes it a constant offset from the Julian Day (JD); however, the
//! MJD is useful because it is not fractional for time points at midnight.

use core::ops::{Add, Sub};

use crate::{
    Convert, Date, Days, Duration, Month,
    errors::{InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate},
    units::SecondsPerDay,
};

/// The Modified Julian Day (MJD) representation of any given date.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifiedJulianDate<Representation, Period = SecondsPerDay> {
    time_since_epoch: Duration<Representation, Period>,
}

/// The modified Julian date of the Unix epoch is useful as constant in some calculations.
const MODIFIED_JULIAN_DATE_UNIX_EPOCH: Days<i32> = Days::new(40587);

impl<Representation, Period> ModifiedJulianDate<Representation, Period> {
    /// Constructs a new MJD directly from some duration since the MJD epoch, November 17 1858.
    pub const fn from_time_since_epoch(time_since_epoch: Duration<Representation, Period>) -> Self {
        Self { time_since_epoch }
    }

    /// Returns the time since the MJD epoch of this day.
    pub const fn time_since_epoch(&self) -> Duration<Representation, Period>
    where
        Representation: Copy,
    {
        self.time_since_epoch
    }

    /// Constructs a modified Julian date from some given calendar date.
    pub fn from_date(date: Date<Representation>) -> Self
    where
        Representation: Copy
            + From<i32>
            + Add<Representation, Output = Representation>
            + Convert<SecondsPerDay, Period>,
    {
        Self {
            time_since_epoch: date.time_since_epoch().into_unit()
                + MODIFIED_JULIAN_DATE_UNIX_EPOCH.cast().into_unit(),
        }
    }

    /// Converts this modified Julian date into the equivalent "universal" calendar date.
    pub fn to_date(&self) -> Date<Representation>
    where
        Representation: Copy
            + From<i32>
            + Sub<Representation, Output = Representation>
            + Convert<Period, SecondsPerDay>,
    {
        Date::from_time_since_epoch(
            self.time_since_epoch.into_unit() - MODIFIED_JULIAN_DATE_UNIX_EPOCH.cast(),
        )
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> ModifiedJulianDate<Target, Period>
    where
        Representation: Into<Target>,
    {
        ModifiedJulianDate::from_time_since_epoch(self.time_since_epoch.cast())
    }

    /// Converts towards a different representation. If the underlying representation cannot store
    /// the result of this cast, returns `None`.
    pub fn try_cast<Target>(
        self,
    ) -> Result<ModifiedJulianDate<Target, Period>, <Representation as TryInto<Target>>::Error>
    where
        Representation: TryInto<Target>,
    {
        Ok(ModifiedJulianDate::from_time_since_epoch(
            self.time_since_epoch.try_cast()?,
        ))
    }
}

impl ModifiedJulianDate<i32> {
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

/// Verifies this implementation by computing the `ModifiedJulianDate` for some known (computed
/// manually or obtained elsewhere) time stamp. If it doesn't match the given `time_since_epoch`,
/// panics.
#[cfg(test)]
fn check_historic_modified_julian_date(
    year: i32,
    month: Month,
    day: u8,
    time_since_epoch: Days<i32>,
) {
    assert_eq!(
        ModifiedJulianDate::from_historic_date(year, month, day)
            .unwrap()
            .time_since_epoch(),
        time_since_epoch,
    );
}

/// Compares some computed MJD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::Month::*;
    check_historic_modified_julian_date(2000, January, 1, Days::new(51544));
    check_historic_modified_julian_date(1999, January, 1, Days::new(51179));
    check_historic_modified_julian_date(1987, January, 27, Days::new(46822));
    check_historic_modified_julian_date(1987, June, 19, Days::new(46965));
    check_historic_modified_julian_date(1988, January, 27, Days::new(47187));
    check_historic_modified_julian_date(1988, June, 19, Days::new(47331));
    check_historic_modified_julian_date(1900, January, 1, Days::new(15020));
    check_historic_modified_julian_date(1600, January, 1, Days::new(-94553));
    check_historic_modified_julian_date(1600, December, 31, Days::new(-94188));
    check_historic_modified_julian_date(837, April, 10, Days::new(-373129));
    check_historic_modified_julian_date(-123, December, 31, Days::new(-723504));
    check_historic_modified_julian_date(-122, January, 1, Days::new(-723503));
    check_historic_modified_julian_date(-1000, July, 12, Days::new(-1044000));
    check_historic_modified_julian_date(-1000, February, 29, Days::new(-1044134));
    check_historic_modified_julian_date(-1001, August, 17, Days::new(-1044330));
    check_historic_modified_julian_date(-4712, January, 1, Days::new(-2400001));
}

/// In practical astrodynamical calculations, it is often useful to be able to create a modified
/// Julian date directly from some known calendar date. However, such calculations must generally
/// be done in floating point arithmetic: this test shows that it is possible to straightforwardly
/// create a float MJD value from some calendar date using a single `cast()`.
#[test]
fn float_mjd_from_date() {
    let mjd = ModifiedJulianDate::from_historic_date(1997, Month::April, 20)
        .unwrap()
        .cast();
    let time_since_epoch = mjd.time_since_epoch();
    assert_eq!(time_since_epoch, Days::new(50558.0f64));
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a MJD based on a date never panics, assuming that the input
    /// date is between validity bounds based on `i32` limits.
    #[kani::proof]
    fn from_date_never_panics() {
        let date: Date<i32> = kani::any();
        kani::assume(
            date.time_since_epoch().count() <= i32::MAX - MODIFIED_JULIAN_DATE_UNIX_EPOCH.count(),
        );
        let _ = ModifiedJulianDate::<i32>::from_date(date);
    }
}
