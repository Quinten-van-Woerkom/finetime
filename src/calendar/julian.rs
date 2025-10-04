//! Implementation of all functionality related to computations regarding the proleptic Julian
//! calendar.

use crate::{Date, Month, duration::Days, errors::InvalidJulianDate};

/// Representation of a proleptic Julian date. Only represents logic down to single-day
/// accuracy: i.e., leap days are included, but leap seconds are not. This is useful in keeping
/// this calendar applicable to all different time scales. Can represent years from -2^31 up to
/// 2^31 - 1.
///
/// This is the calendar effectively used by the `hifitime` and `chrono` libraries.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JulianDate {
    year: i32,
    month: Month,
    day: u8,
}

impl JulianDate {
    /// Creates a new Julian date, given its `year`, `month`, and `day`. If the date is not a
    /// valid proleptic Julian date, returns a `JulianDateDoesNotExist` to indicate that the
    /// requested date does not exist in the proleptic Julian calendar.
    ///
    /// This function will never panic.
    pub const fn new(year: i32, month: Month, day: u8) -> Result<Self, InvalidJulianDate> {
        if Self::is_valid_date(year, month, day) {
            Ok(Self { year, month, day })
        } else {
            Err(InvalidJulianDate { year, month, day })
        }
    }

    /// Constructs a Julian date from a given `Date<i32>` instance. Useful primarily when an
    /// existing `Date` must be printed in human-readable format.
    ///
    /// Uses Howard Hinnant's `julian_from_days` algorithm.
    pub const fn from_date(date: Date<i32>) -> Self {
        let days = date.time_since_epoch().count();
        // Shift epoch from 1970-01-01 to 0000-03-01
        let z = days + 719470;

        let era = if z >= 0 { z } else { z - 1460 } / 1461;
        let doe = z - era * 1461; // [0, 1461]
        let yoe = (doe - doe / 1460) / 365; // [0, 3]
        let year = yoe + era * 4;
        let doy = doe - 365 * yoe; // [0, 365]
        let mp = (5 * doy + 2) / 153; // [0, 11]
        let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
        let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
        let year = if month <= 2 { year + 1 } else { year };
        let month = match Month::try_from(month as u8) {
            Ok(month) => month,
            Err(_) => unreachable!(),
        };
        let day = day as u8;

        Self { year, month, day }
    }

    /// Constructs a `Date` from a given Julian date. Uses Howard Hinnant's `days_from_julian`
    /// algorithm.
    pub const fn to_date(&self) -> Date<i32> {
        let mut year = self.year;
        let month = self.month as i32;
        let day = self.day as i32;
        if month <= 2 {
            year -= 1;
        }
        let era = if year >= 0 { year } else { year - 3 } / 4;
        let yoe = year - era * 4;
        let doy = (153 * if month > 2 { month - 3 } else { month + 9 } + 2) / 5 + day - 1;
        let doe = yoe * 365 + doy;
        let days_since_epoch = era * 1461 + doe - 719470;
        let time_since_epoch = Days::new(days_since_epoch);
        Date::from_time_since_epoch(time_since_epoch)
    }

    /// Returns the year stored inside this proleptic Julian date. Astronomical year
    /// numbering is used (as also done in NAIF SPICE): the year 1 BCE is represented as 0, 2 BCE as
    /// -1, etc. Hence, around the year 0, the numbering is ..., -2 (3 BCE), -1 (2 BCE), 0 (1 BCE),
    /// 1 (1 CE), 2 (2 CE), et cetera. In this manner, the year numbering proceeds smoothly through 0.
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month stored inside this proleptic Julian date.
    pub const fn month(&self) -> Month {
        self.month
    }

    /// Returns the day-of-month stored inside this proleptic Julian date.
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Returns the number of days in a given month of a year.
    const fn days_in_month(year: i32, month: Month) -> u8 {
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

    /// Returns whether the given calendar year is a leap year or not.
    const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0
    }

    /// Returns whether the given calendar date is a valid proleptic Julian calendar date.
    /// This includes whenever the day does not exist in a given year-month combination.
    const fn is_valid_date(year: i32, month: Month, day: u8) -> bool {
        day != 0 && day <= Self::days_in_month(year, month)
    }
}

impl From<JulianDate> for Date<i32> {
    fn from(value: JulianDate) -> Self {
        value.to_date()
    }
}

impl From<Date<i32>> for JulianDate {
    fn from(value: Date<i32>) -> Self {
        Self::from_date(value)
    }
}

/// Verifies that roundtrip conversion for some random dates conserves the date.
#[test]
fn roundtrip() {
    // We check some simple and edge case timestamps.
    let times_since_epoch = [
        Days::new(42),
        Days::new(719470),
        Days::new(-42i32),
        Days::new(-719470),
        Days::new(i32::MAX - 719470),
        Days::new(i32::MIN),
    ];

    for time_since_epoch in times_since_epoch.iter() {
        let date = Date::from_time_since_epoch(*time_since_epoch);
        let julian_date = JulianDate::from_date(date);
        let date2 = julian_date.to_date();
        let julian_date2 = JulianDate::from_date(date2);
        assert_eq!(date, date2);
        assert_eq!(julian_date, julian_date2);
    }

    // Afterwards, we verify 10_000 uniformly distributed random numbers
    use rand::prelude::*;
    let mut rng = rand::rng();
    for _ in 0..10000 {
        let days_since_epoch = rng.random::<i32>() % (i32::MAX - 719470);
        let time_since_epoch = Days::new(days_since_epoch);
        let date = Date::from_time_since_epoch(time_since_epoch);
        let julian_date = JulianDate::from_date(date);
        let date2 = julian_date.to_date();
        let julian_date2 = JulianDate::from_date(date2);
        assert_eq!(date, date2);
        assert_eq!(julian_date, julian_date2);
    }

    // And finally, we check this property for all days in the years 0 to 3000.
    for year in 0..3000 {
        for month in 1..=12 {
            for day in 1..=31 {
                let month = match month {
                    1u8 => Month::January,
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
                    _ => unreachable!(),
                };

                if let Ok(julian_date) = JulianDate::new(year, month, day) {
                    let date = julian_date.to_date();
                    let julian_date2 = JulianDate::from_date(date);
                    assert_eq!(julian_date, julian_date2);
                }
            }
        }
    }
}

#[cfg(kani)]
impl kani::Arbitrary for JulianDate {
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

    /// Verifies that construction of a Julian date never panics.
    #[kani::proof]
    fn construction_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let _ = JulianDate::new(year, month, day);
    }

    /// Verifies that conversion of a Julian date into a "universal" date never panics for all
    /// values within a well-defined range.
    #[kani::proof]
    fn conversion_to_date_never_panics() {
        let julian_date: JulianDate = kani::any();
        kani::assume(julian_date >= JulianDate::new(-5877520, Month::March, 3).unwrap());
        kani::assume(julian_date <= JulianDate::new(5879489, Month::December, 16).unwrap());
        let _ = julian_date.to_date();
    }

    /// Verifies that conversion from a "universal" date into a Julian date never panics for all
    /// values within a well-defined range.
    #[kani::proof]
    fn conversion_from_date_never_panics() {
        let date: Date<i32> = kani::any();
        kani::assume(date <= Date::from_time_since_epoch(Days::new(i32::MAX - 719470)));
        let _ = JulianDate::from_date(date);
    }
}
