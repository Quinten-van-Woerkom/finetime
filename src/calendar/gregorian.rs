//! Implementation of all functionality related to computations regarding the Gregorian calendar.
//! This is also often referred to as the civil calendar, based on it being the civil calendar for
//! most countries.

use core::ops::Sub;

use crate::{
    GregorianDateDoesNotExist, calendar::Month, duration::Days, time_point::TimePoint,
    time_scale::LocalDays,
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

    /// Constructs a MJD from a given Gregorian date. Applies a slight variation on the approach
    /// described by Meeus in Astronomical Algorithms (Chapter 7, Julian Day). This variation
    /// adapts the algorithm to the Unix epoch, and removes the dependency on floating point
    /// arithmetic.
    pub const fn to_local_days(self) -> LocalDays<i64> {
        let (mut year, mut month, day) =
            (self.year() as i64, self.month() as i64, self.day() as i64);
        if month <= 2 {
            year -= 1;
            month += 12;
        }

        // Applies the leap year correction, as described in Meeus.
        let leap_year_correction = {
            let a = year.div_euclid(100);
            2 - a + a / 4
        };

        // Computes the days because of elapsed years. Equivalent to `INT(365.25(Y + 4716))` from
        // Meeus.
        let year_days = (365 * (year + 4716)) + (year + 4716) / 4;

        // Computes the days due to elapsed months. Equivalent to `INT(30.6001(M + 1))` from Meeus.
        let month_days = (306001 * (month + 1)) / 10000;

        // Computes the Julian day number following Meeus' approach - though as an integer with an
        // offset of 0.5 days. Then, we subtract 2440587.5 (on top of Meeus' 1524.5) to obtain the
        // time since the Unix epoch.
        let days_since_epoch = year_days + month_days + day + leap_year_correction - 2442112;
        TimePoint::from_time_since_epoch(Days::new(days_since_epoch))
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

impl From<GregorianDate> for LocalDays<i64> {
    fn from(value: GregorianDate) -> Self {
        value.to_local_days()
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
        let days_lhs = self.to_local_days();
        let days_rhs = rhs.to_local_days();
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
