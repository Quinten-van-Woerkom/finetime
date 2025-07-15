//! Implementation of all functionality related to computations regarding the Gregorian calendar.
//! This is also often referred to as the civil calendar, based on it being the civil calendar for
//! most countries.

use core::ops::Sub;

use crate::{
    calendar::{Datelike, Month},
    duration::Days,
    time_scale::local::{LocalDays, LocalTime},
};

/// Representation of a proleptic Gregorian date. Only represents logic down to single-day
/// accuracy: i.e., leap days are included, but leap seconds are not. This is useful in keeping
/// this calendar applicable to all different time scales. Can represent years from -2^31 up to
/// 2^31 - 1.
///
/// This is the calendar effectively used by the `hifitime` and `chrono` libraries.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GregorianDate {
    year: i32,
    month: Month,
    day: u8,
}

impl GregorianDate {
    /// Creates a new Gregorian date, given its `year`, `month`, and `day`. If the date is not a
    /// valid proleptic Gregorian date, returns a `GregorianDateDoesNotExist` to indicate that the
    /// requested date does not exist in the proleptic Gregorian calendar.
    ///
    /// This function will never panic.
    pub const fn new(year: i32, month: Month, day: u8) -> Result<Self, GregorianDateDoesNotExist> {
        if Self::is_valid_date(year, month, day) {
            Ok(Self { year, month, day })
        } else {
            Err(GregorianDateDoesNotExist { year, month, day })
        }
    }

    /// Returns the year stored inside this proleptic Gregorian date. Astronomical year
    /// numbering is used (as also done in NAIF SPICE): the year 1 BCE is represented as 0, 2 BCE as
    /// -1, etc. Hence, around the year 0, the numbering is ..., -2 (3 BCE), -1 (2 BCE), 0 (1 BCE),
    /// 1 (1 CE), 2 (2 CE), et cetera. In this manner, the year numbering proceeds smoothly through 0.
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month stored inside this proleptic Gregorian date.
    pub const fn month(&self) -> Month {
        self.month
    }

    /// Returns the day-of-month stored inside this proleptic Gregorian date.
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Returns the number of days in a given month of a year.
    const fn days_in_month(year: i32, month: Month) -> u8 {
        use crate::calendar::Month::*;
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

    /// Returns whether the given calendar year is a leap year or not.
    const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// Returns whether the given calendar date is a valid proleptic Gregorian calendar date.
    /// This includes whenever the day does not exist in a given year-month combination.
    const fn is_valid_date(year: i32, month: Month, day: u8) -> bool {
        day != 0 && day <= Self::days_in_month(year, month)
    }
}

impl Datelike for GregorianDate {}

impl From<GregorianDate> for LocalDays<i64> {
    fn from(value: GregorianDate) -> Self {
        LocalDays::from_gregorian_date(value)
    }
}

impl Sub for GregorianDate {
    type Output = Days<i64>;

    /// The difference between two Gregorian dates can be computed exactly as a number of days,
    /// accounting for the variable number of days per leap year. Note that this is only possible
    /// up to an accuracy of days because leap seconds depend on the time scale.
    ///
    /// An intermediate MJD representation is used for this, because subtracting two MJDs is very
    /// cheap to do.
    fn sub(self, rhs: Self) -> Self::Output {
        let days_lhs = LocalTime::from_gregorian_date(self);
        let days_rhs = LocalTime::from_gregorian_date(rhs);
        days_lhs - days_rhs
    }
}

#[cfg(kani)]
impl kani::Arbitrary for GregorianDate {
    fn any() -> Self {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let mut day: u8 = kani::any::<u8>() % 32u8;
        if !Self::is_valid_date(year, month, day) {
            day = 1;
        }
        Self { year, month, day }
    }
}

/// Error returned when the requested Gregorian date does not exist, because the requested
/// combination of month and year does not have the requested number of days.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GregorianDateDoesNotExist {
    year: i32,
    month: Month,
    day: u8,
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a Gregorian date never panics.
    #[kani::proof]
    fn construction_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let _ = GregorianDate::new(year, month, day);
    }
}
