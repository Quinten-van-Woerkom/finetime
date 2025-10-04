//! Representation of some calendrical time point as the elapsed number of (potentially fractional)
//! days since the start of the Julian period.

use core::ops::{Add, Sub};

use crate::{
    Convert, Date, Days, Duration, HalfDays, InvalidGregorianDate, InvalidHistoricDate,
    InvalidJulianDate, Month,
    units::{SecondsPerDay, SecondsPerHalfDay},
};

/// Representation of calendrical dates in terms of Julian Days (JD). A Julian day is the number
/// of elapsed days since the start of the Julian period: noon on January 1, 4713 BC in the
/// proleptic Julian calendar, or equivalently, on November 24, 4714 BC in the proleptic Gregorian
/// calendar.
///
/// For convenience, also supports instants that are expressed in other units than days. This can
/// be convenient, for example, when one needs to represents a fractional time in an exact
/// (non-float) manner. However, in such cases a second is not actually an SI second but rather
/// 1/86,400 of that Julian day. This distinction may be important when the represented day
/// contains a leap second.
///
/// It must be noted that this time representation does not contain an associated time scale, so it
/// is actually ambiguous. Indeed, it may only indicate a calendrical date, but not an actual point
/// in time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Constructor)]
pub struct JulianDay<Representation, Period = SecondsPerHalfDay> {
    day: Duration<Representation, Period>,
}

impl<Representation, Period> JulianDay<Representation, Period> {
    /// Constructs a Julian day from some given calendar date.
    pub fn from_date(date: Date<Representation>) -> Self
    where
        Representation: Copy
            + From<i32>
            + Add<Representation, Output = Representation>
            + Convert<SecondsPerDay, Period>
            + Convert<SecondsPerHalfDay, Period>,
    {
        Self {
            day: date.time_since_epoch().into_unit()
                + Days::new(2440587i32).cast().into_unit()
                + HalfDays::new(1i32).cast().into_unit(),
        }
    }

    pub fn to_date(&self) -> Date<Representation>
    where
        Representation: Copy
            + From<i32>
            + Sub<Representation, Output = Representation>
            + Convert<Period, SecondsPerDay>
            + Convert<SecondsPerHalfDay, SecondsPerDay>,
    {
        Date::from_time_since_epoch(
            self.day.into_unit()
                - Days::new(2440587i32).cast()
                - HalfDays::new(1i32).cast().into_unit(),
        )
    }
}

impl JulianDay<i64, SecondsPerHalfDay> {
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

impl<Representation, Period> From<JulianDay<Representation, Period>> for Date<Representation>
where
    Representation: Copy
        + From<i32>
        + Sub<Representation, Output = Representation>
        + Convert<Period, SecondsPerDay>
        + Convert<SecondsPerHalfDay, SecondsPerDay>,
{
    fn from(value: JulianDay<Representation, Period>) -> Self {
        value.to_date()
    }
}

impl<Representation, Period> From<Date<Representation>> for JulianDay<Representation, Period>
where
    Representation: Copy
        + From<i32>
        + Add<Representation, Output = Representation>
        + Convert<SecondsPerDay, Period>
        + Convert<SecondsPerHalfDay, Period>,
{
    fn from(value: Date<Representation>) -> Self {
        Self::from_date(value)
    }
}

/// Compares some computed JD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::Month::*;
    assert_eq!(
        JulianDay::from_historic_date(2000, January, 1).unwrap(),
        JulianDay::new(Days::new(2451544).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1999, January, 1).unwrap(),
        JulianDay::new(Days::new(2451179).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1987, January, 27).unwrap(),
        JulianDay::new(Days::new(2446822).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1987, June, 19).unwrap(),
        JulianDay::new(Days::new(2446965).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1988, January, 27).unwrap(),
        JulianDay::new(Days::new(2447187).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1988, June, 19).unwrap(),
        JulianDay::new(Days::new(2447331).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1900, January, 1).unwrap(),
        JulianDay::new(Days::new(2415020).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1600, January, 1).unwrap(),
        JulianDay::new(Days::new(2305447).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(1600, December, 31).unwrap(),
        JulianDay::new(Days::new(2305812).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(837, April, 10).unwrap(),
        JulianDay::new(Days::new(2026871).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-123, December, 31).unwrap(),
        JulianDay::new(Days::new(1676496).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-122, January, 1).unwrap(),
        JulianDay::new(Days::new(1676497).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-1000, July, 12).unwrap(),
        JulianDay::new(Days::new(1356000).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-1000, February, 29).unwrap(),
        JulianDay::new(Days::new(1355866).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-1001, August, 17).unwrap(),
        JulianDay::new(Days::new(1355670).into_unit() + HalfDays::new(1i64))
    );
    assert_eq!(
        JulianDay::from_historic_date(-4712, January, 1).unwrap(),
        JulianDay::new(-HalfDays::new(1i64))
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
        let _ = JulianDay::<i64>::from_date(date.to_date());
    }

    /// Verifies that construction of a MJD based on a Gregorian date never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::GregorianDate;
        let date: GregorianDate = kani::any();
        let _ = JulianDay::<i64>::from_date(date.to_date());
    }
}
