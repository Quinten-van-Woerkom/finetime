//! Implementation of the historic or "civil" calendar.

//! Implementation of the historic calendar, which is Julian before and Gregorian after the
//! Gregoric calendar reform of 1582. When in doubt, use this calendar.

use crate::{
    Date, GregorianDate, JulianDate, Month,
    errors::{InvalidDayOfYear, InvalidDayOfYearCount, InvalidHistoricDate},
};

/// Implementation of a date in the historic calendar. After 15 October 1582, this coincides with
/// the Gregorian calendar; until 4 October 1582, this is the Julian calendar. The days inbetween
/// do not exist.
///
/// This is the calendar that is also used by IAU SOFA and NAIF SPICE, as well as Meeus in his
/// Astronomical Algorithms book. Hence, most users probably expect it to be the calendar of
/// choice.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HistoricDate {
    year: i32,
    month: Month,
    day: u8,
}

impl HistoricDate {
    /// Creates a new date, given its `year`, `month`, and `day`. If the date is not a valid date
    /// in the historic calendar, returns a `DateDoesNotExist` error to indicate that the
    /// requested date does not exist.
    ///
    /// This function will never panic.
    pub const fn new(year: i32, month: Month, day: u8) -> Result<Self, InvalidHistoricDate> {
        if Self::is_valid_date(year, month, day) {
            Ok(Self { year, month, day })
        } else {
            Err(InvalidHistoricDate { year, month, day })
        }
    }

    /// Creates a new date given only the year and the day-of-year. Implementation is based on an
    /// algorithm found by A. Pouplier and reported by Jean Meeus in Astronomical Algorithms.
    ///
    /// This function will never panic.
    pub const fn from_ordinal_date(year: i32, day_of_year: u16) -> Result<Self, InvalidDayOfYear> {
        let is_leap_year = Self::is_leap_year(year);
        let (month, day) = match month_day_from_ordinal_date(year, day_of_year, is_leap_year) {
            Ok((month, day)) => (month, day),
            Err(error) => return Err(error),
        };

        // It is still possible for a date to have been computed that is part of the Gregorian
        // calendar reform period (5 October up to and including 14 October 1582). We must reject
        // such dates in the historic calendar.
        match Self::new(year, month, day) {
            Ok(date) => Ok(date),
            Err(err) => Err(InvalidDayOfYear::InvalidHistoricDate(err)),
        }
    }

    pub const fn from_date(date: Date<i32>) -> Self {
        // Determine which calendar applies: Julian or Gregorian
        const GREGORIAN_REFORM: Date<i32> = match GregorianDate::new(1582, Month::October, 15) {
            Ok(date) => date.to_date(),
            Err(_) => unreachable!(),
        };
        let is_gregorian =
            date.time_since_epoch().count() >= GREGORIAN_REFORM.time_since_epoch().count();

        if is_gregorian {
            let date = GregorianDate::from_date(date);
            Self {
                year: date.year(),
                month: date.month(),
                day: date.day(),
            }
        } else {
            let date = JulianDate::from_date(date);
            Self {
                year: date.year(),
                month: date.month(),
                day: date.day(),
            }
        }
    }

    /// Constructs a generic date from a given historic calendar date. Applies a slight variation
    /// on the approach described by Meeus in Astronomical Algorithms (Chapter 7, Julian Day). This
    /// variation adapts the algorithm to the Unix epoch and removes the dependency on floating
    /// point arithmetic.
    pub const fn to_date(self) -> Date<i32> {
        let HistoricDate { year, month, day } = self;
        if self.is_gregorian() {
            match GregorianDate::new(year, month, day) {
                Ok(date) => date.to_date(),
                Err(_) => unreachable!(),
            }
        } else {
            match JulianDate::new(year, month, day) {
                Ok(date) => date.to_date(),
                Err(_) => unreachable!(),
            }
        }
    }

    /// Returns the year stored inside this historic date. Astronomical year numbering is used (as
    /// also done in NAIF SPICE): the year 1 BCE is represented as 0, 2 BCE as -1, etc. Hence,
    /// around the year 0, the numbering is ..., -2 (3 BCE), -1 (2 BCE), 0 (1 BCE), 1 (1 CE), 2 (2
    /// CE), et cetera. In this manner, the year numbering proceeds smoothly through 0.
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month stored inside this historic date.
    pub const fn month(&self) -> Month {
        self.month
    }

    /// Returns the day-of-month stored inside this historic date.
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Returns the day-of-year of this specific date, within its calendar year. The day-of-year is
    /// an integer value ranging from 1 on January 1 to 365 (or 365, in leap years) on December 31.
    /// Uses the algorithm given by Meeus in Astronomical Algorithms.
    pub const fn day_of_year(&self) -> u16 {
        let k = if Self::is_leap_year(self.year) { 1 } else { 2 };
        let m = self.month() as u16;
        let d = self.day() as u16;
        ((275 * m) / 9) - k * ((m + 9) / 12) + d - 30
    }

    /// Returns whether the current date falls within the Gregorian (true) or Julian (false) part
    /// of the historic calendar.
    pub const fn is_gregorian(&self) -> bool {
        self.year > 1582
            || (self.year == 1582
                && (self.month as u8 > Month::October as u8
                    || (self.month as u8 == Month::October as u8 && self.day >= 15)))
    }

    /// Returns the number of days in a given month of a year. Also considers whether the given
    /// year-month combination would fall in the Gregorian or Julian calendar.
    pub const fn days_in_month(year: i32, month: Month) -> u8 {
        use crate::Month::*;
        match month {
            January | March | May | July | August | October | December => 31,
            April | June | September | November => 30,
            February => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
        }
    }

    /// Returns whether the given calendar year is a leap year or not. Because of the Gregorian
    /// calendar reform, this differs depending on whether the date is after 1582 or before.
    const fn is_leap_year(year: i32) -> bool {
        if year > 1582 {
            (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
        } else {
            year % 4 == 0
        }
    }

    /// Returns whether the given calendar date is a valid historic calendar date.
    const fn is_valid_date(year: i32, month: Month, day: u8) -> bool {
        day != 0
            && day <= Self::days_in_month(year, month)
            && !Self::falls_during_gregorian_reform(year, month, day)
    }

    /// Returns whether the given calendar date falls within the Gregorian calendar reform period,
    /// which is a set of 10 days that were skipped during the reform. The day after 4 October 1582
    /// in the historic calendar is 15 October 1582.
    const fn falls_during_gregorian_reform(year: i32, month: Month, day: u8) -> bool {
        year == 1582 && month as u8 == Month::October as u8 && day > 4 && day < 15
    }
}

/// It turns out that the `from_ordinal_date` implementation can largely be factored into one
/// function that is valid for both the historic, proleptic Gregorian, and proleptic Julian
/// calendars. After all, the function depends only on whether or not some year is a leap year.
pub(crate) const fn month_day_from_ordinal_date(
    year: i32,
    day_of_year: u16,
    is_leap_year: bool,
) -> Result<(Month, u8), InvalidDayOfYear> {
    if day_of_year == 0 || day_of_year > 366 || (day_of_year == 366 && !is_leap_year) {
        return Err(InvalidDayOfYear::InvalidDayOfYearCount(
            InvalidDayOfYearCount { year, day_of_year },
        ));
    }

    // Compute the month and day-of-month.
    let k = if is_leap_year { 1 } else { 2 };
    let month = if day_of_year < 32 {
        1
    } else {
        (9 * (k + day_of_year as i32) + 269) / 275
    };
    let day = day_of_year as i32 - (275 * month) / 9 + k * ((month + 9) / 12) + 30;

    // Validate the output range. This should not actually fail, but we need to handle it for
    // casting to the proper output types.
    let day = match day {
        0..=32 => day as u8,
        _ => unreachable!(),
    };
    let month = match Month::try_from(month as u8) {
        Ok(month) => month,
        Err(_) => unreachable!(),
    };
    Ok((month, day))
}

impl From<HistoricDate> for Date<i32> {
    fn from(value: HistoricDate) -> Self {
        value.to_date()
    }
}

impl From<Date<i32>> for HistoricDate {
    fn from(value: Date<i32>) -> Self {
        Self::from_date(value)
    }
}

/// Tests the day-of-year function using some examples from Meeus.
#[test]
fn day_of_year() {
    // Computing the day-of-year based on some date.
    let date1 = HistoricDate::new(1978, Month::November, 14).unwrap();
    assert_eq!(date1.day_of_year(), 318);
    let date2 = HistoricDate::new(1988, Month::April, 22).unwrap();
    assert_eq!(date2.day_of_year(), 113);

    // The reverse procedure: computing the date based on a year and day-of-year.
    let date3 = HistoricDate::from_ordinal_date(1978, 318).unwrap();
    assert_eq!(date3, date1);
    let date4 = HistoricDate::from_ordinal_date(1988, 113).unwrap();
    assert_eq!(date4, date2);
}

/// Verifies that the Gregorian calendar reform is properly modelled.
#[test]
fn gregorian_reform() {
    use crate::Days;
    use crate::Month::*;
    let date1 = Date::from_historic_date(1582, October, 4).unwrap();
    let date2 = Date::from_historic_date(1582, October, 15).unwrap();
    assert_eq!(date1 + Days::new(1), date2);
}

#[cfg(kani)]
impl kani::Arbitrary for HistoricDate {
    fn any() -> Self {
        let mut year: i32 = kani::any();
        let month: Month = kani::any();
        let mut day: u8 = kani::any::<u8>() % 32u8;
        if !Self::is_valid_date(year, month, day) {
            // The date may be invalid either because the day is not a valid day for a given month,
            // or because the date falls in the date window skipped by the Gregorian calendar
            // reform. Both cases can be handled by setting the day and year both to 1.
            day = 1;
            year = 1;
        }
        Self { year, month, day }
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a historic date never panics.
    #[kani::proof]
    fn construction_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let _ = HistoricDate::new(year, month, day);
    }

    /// Verifies that construction of a historic date from a year and day-of-year never panics,
    /// also not on invalid inputs.
    #[kani::proof]
    fn day_of_year_never_panics() {
        let year: i32 = kani::any();
        let day_of_year: u16 = kani::any();
        let _ = HistoricDate::from_ordinal_date(year, day_of_year);
    }

    /// Verifies that, for any correct date, computing its day-of-year and using that to
    /// reconstruct the date, will not panic and will result in the exact same value.
    #[kani::proof]
    fn day_of_year_roundtrip() {
        let date: HistoricDate = kani::any();
        let year = date.year();
        let day_of_year = date.day_of_year();
        let reconstructed = HistoricDate::from_ordinal_date(year, day_of_year).unwrap();
        assert_eq!(date, reconstructed);
    }

    /// Verifies that conversion to and from a `Date` is well-defined for all possible values of
    /// `Date<i32>`: no panics, undefined behaviour, or arithmetic errors.
    #[kani::proof]
    fn date_conversion_well_defined() {
        let date: Date<i32> = kani::any();
        let historic_date = HistoricDate::from_date(date);
        let _ = historic_date.to_date();
    }
}
