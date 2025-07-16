//! The Modified Julian Day (MJD) is nothing more than the number of days since 1858 November 17
//! at 0h UT. Effectively, this makes it a constant offset from the Julian Day (JD); however, the
//! MJD is useful because it is not fractional for time points at midnight.

use core::ops::{Add, Sub};

use crate::{duration::Days, time_scale::local::LocalDays};

/// The Modified Julian Day (MJD) representation of any given date.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModifiedJulianDay<T> {
    day: Days<T>,
}

impl<T> ModifiedJulianDay<T> {
    /// Constructs a new MJD directly from some day number.
    pub const fn new(day: Days<T>) -> Self {
        Self { day }
    }
}

impl ModifiedJulianDay<i64> {
    /// Constructs a MJD from a given calendar date.
    pub fn from_date(date: impl Into<LocalDays<i64>>) -> Self {
        let local_days = date.into();
        Self {
            day: local_days.elapsed_time_since_epoch() + Days::new(40587),
        }
    }
}

impl From<LocalDays<i64>> for ModifiedJulianDay<i64> {
    /// Transforming from `LocalDays` (since Unix epoch) to the equivalent `ModifiedJulianDay` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: LocalDays<i64>) -> Self {
        Self {
            day: value.elapsed_time_since_epoch() + Days::new(40587),
        }
    }
}

impl From<ModifiedJulianDay<i64>> for LocalDays<i64> {
    /// Transforming to `LocalDays` (since Unix epoch) from the equivalent `ModifiedJulianDay` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: ModifiedJulianDay<i64>) -> Self {
        Self::from_time_since_epoch(value.day - Days::new(40587))
    }
}

/// Compares some computed MJD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::calendar::Date;
    use crate::calendar::Month::*;
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(2000, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(51544))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1999, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(51179))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1987, January, 27).unwrap()),
        ModifiedJulianDay::new(Days::new(46822))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1987, June, 19).unwrap()),
        ModifiedJulianDay::new(Days::new(46965))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1988, January, 27).unwrap()),
        ModifiedJulianDay::new(Days::new(47187))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1988, June, 19).unwrap()),
        ModifiedJulianDay::new(Days::new(47331))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1900, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(15020))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1600, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(-94553))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(1600, December, 31).unwrap()),
        ModifiedJulianDay::new(Days::new(-94188))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(837, April, 10).unwrap()),
        ModifiedJulianDay::new(Days::new(-373129))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-123, December, 31).unwrap()),
        ModifiedJulianDay::new(Days::new(-723504))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-122, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(-723503))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-1000, July, 12).unwrap()),
        ModifiedJulianDay::new(Days::new(-1044000))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-1000, February, 29).unwrap()),
        ModifiedJulianDay::new(Days::new(-1044134))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-1001, August, 17).unwrap()),
        ModifiedJulianDay::new(Days::new(-1044330))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(Date::new(-4712, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(-2400001))
    );
}

/// Compares some computed MJD values with known values from Meeus' Astronomical Algorithms. Note
/// that Meeus switches to the Julian calendar in dates preceding the Gregorian reform (i.e., prior
/// to 15 October 1582). Hence, we only consider dates after this reform.
#[test]
fn gregorian_dates_from_meeus() {
    use crate::calendar::GregorianDate;
    use crate::calendar::Month::*;
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(2000, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(51544))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1999, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(51179))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1987, January, 27).unwrap()),
        ModifiedJulianDay::new(Days::new(46822))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1987, June, 19).unwrap()),
        ModifiedJulianDay::new(Days::new(46965))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1988, January, 27).unwrap()),
        ModifiedJulianDay::new(Days::new(47187))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1988, June, 19).unwrap()),
        ModifiedJulianDay::new(Days::new(47331))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1900, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(15020))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1600, January, 1).unwrap()),
        ModifiedJulianDay::new(Days::new(-94553))
    );
    assert_eq!(
        ModifiedJulianDay::from_date(GregorianDate::new(1600, December, 31).unwrap()),
        ModifiedJulianDay::new(Days::new(-94188))
    );
}

impl<T> Sub for ModifiedJulianDay<T>
where
    T: Sub<Output = T>,
{
    type Output = Days<T>;

    /// The MJD representation of dates is very useful in that it permits direct computations with
    /// the underlying values, because those are nothing more than continuous day counts.
    fn sub(self, rhs: Self) -> Self::Output {
        self.day - rhs.day
    }
}

impl<T> Add<Days<T>> for ModifiedJulianDay<T>
where
    T: Add<Output = T>,
{
    type Output = ModifiedJulianDay<T>;

    /// Adding days to a MJD is nothing more than an integer addition.
    fn add(self, rhs: Days<T>) -> Self::Output {
        Self::new(self.day + rhs)
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a MJD based on a historic date never panics.
    #[kani::proof]
    fn from_historic_date_never_panics() {
        use crate::calendar::Date;
        let date: Date = kani::any();
        let _ = ModifiedJulianDay::from_date(date);
    }

    /// Verifies that construction of a MJD based on a Gregorian date never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let _ = ModifiedJulianDay::from_date(date);
    }
}
