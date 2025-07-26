//! The Modified Julian Day (MJD) is nothing more than the number of days since 1858 November 17
//! at 0h UT. Effectively, this makes it a constant offset from the Julian Day (JD); however, the
//! MJD is useful because it is not fractional for time points at midnight.

use core::ops::{Add, Mul, Sub};
use std::ops::Div;

use num::NumCast;

use crate::{
    Duration, LocalTime,
    duration::Days,
    time_scale::LocalDays,
    units::{IsValidConversion, Ratio, SecondsPerDay},
};

/// The Modified Julian Day (MJD) representation of any given date.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModifiedJulianDate<T, Period = SecondsPerDay> {
    day: Duration<T, Period>,
}

impl<T, Period> ModifiedJulianDate<T, Period> {
    /// Constructs a new MJD directly from some duration since the MJD epoch, November 17 1858.
    pub const fn new(duration: Duration<T, Period>) -> Self {
        Self { day: duration }
    }
}

impl ModifiedJulianDate<i64> {
    /// Constructs a MJD from a given calendar date.
    pub fn from_date(date: impl Into<LocalDays<i64>>) -> Self {
        let local_days = date.into();
        Self {
            day: local_days.elapsed_time_since_epoch() + Days::new(40587),
        }
    }
}

impl<Representation, Period> From<LocalTime<Representation, Period>>
    for ModifiedJulianDate<Representation, Period>
where
    Period: Ratio,
    Representation: Copy
        + From<i64>
        + Add<Representation, Output = Representation>
        + Mul<Representation, Output = Representation>
        + Div<Representation, Output = Representation>
        + NumCast,
    (): IsValidConversion<Representation, SecondsPerDay, Period>,
{
    /// Transforming from `LocalDays` (since Unix epoch) to the equivalent `ModifiedJulianDate` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: LocalTime<Representation, Period>) -> Self {
        Self {
            day: value.elapsed_time_since_epoch() + Days::new(40587).cast().convert(),
        }
    }
}

impl<Representation, Period> From<ModifiedJulianDate<Representation, Period>>
    for LocalTime<Representation, Period>
where
    Period: Ratio,
    Representation: Copy
        + From<i64>
        + Sub<Representation, Output = Representation>
        + Mul<Representation, Output = Representation>
        + Div<Representation, Output = Representation>
        + NumCast,
    (): IsValidConversion<Representation, SecondsPerDay, Period>,
{
    /// Transforming to `LocalDays` (since Unix epoch) from the equivalent `ModifiedJulianDate` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: ModifiedJulianDate<Representation, Period>) -> Self {
        Self::from_time_since_epoch(value.day - Days::new(40587).cast().convert())
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
        ModifiedJulianDate::from_date(Date::new(2000, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(51544))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1999, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(51179))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1987, January, 27).unwrap()),
        ModifiedJulianDate::new(Days::new(46822))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1987, June, 19).unwrap()),
        ModifiedJulianDate::new(Days::new(46965))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1988, January, 27).unwrap()),
        ModifiedJulianDate::new(Days::new(47187))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1988, June, 19).unwrap()),
        ModifiedJulianDate::new(Days::new(47331))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1900, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(15020))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1600, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(-94553))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(1600, December, 31).unwrap()),
        ModifiedJulianDate::new(Days::new(-94188))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(837, April, 10).unwrap()),
        ModifiedJulianDate::new(Days::new(-373129))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-123, December, 31).unwrap()),
        ModifiedJulianDate::new(Days::new(-723504))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-122, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(-723503))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-1000, July, 12).unwrap()),
        ModifiedJulianDate::new(Days::new(-1044000))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-1000, February, 29).unwrap()),
        ModifiedJulianDate::new(Days::new(-1044134))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-1001, August, 17).unwrap()),
        ModifiedJulianDate::new(Days::new(-1044330))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(Date::new(-4712, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(-2400001))
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
        ModifiedJulianDate::from_date(GregorianDate::new(2000, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(51544))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1999, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(51179))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1987, January, 27).unwrap()),
        ModifiedJulianDate::new(Days::new(46822))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1987, June, 19).unwrap()),
        ModifiedJulianDate::new(Days::new(46965))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1988, January, 27).unwrap()),
        ModifiedJulianDate::new(Days::new(47187))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1988, June, 19).unwrap()),
        ModifiedJulianDate::new(Days::new(47331))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1900, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(15020))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1600, January, 1).unwrap()),
        ModifiedJulianDate::new(Days::new(-94553))
    );
    assert_eq!(
        ModifiedJulianDate::from_date(GregorianDate::new(1600, December, 31).unwrap()),
        ModifiedJulianDate::new(Days::new(-94188))
    );
}

impl<Representation, Period> Sub for ModifiedJulianDate<Representation, Period>
where
    Representation: Sub<Output = Representation>,
    Period: Ratio,
{
    type Output = Duration<Representation, Period>;

    /// The MJD representation of dates is very useful in that it permits direct computations with
    /// the underlying values, because those are nothing more than continuous day counts.
    fn sub(self, rhs: Self) -> Self::Output {
        self.day - rhs.day
    }
}

impl<Representation, Period> Add<Duration<Representation, Period>>
    for ModifiedJulianDate<Representation, Period>
where
    Representation: Add<Output = Representation>,
    Period: Ratio,
{
    type Output = ModifiedJulianDate<Representation, Period>;

    /// Adding days to a MJD is nothing more than an integer addition.
    fn add(self, rhs: Duration<Representation, Period>) -> Self::Output {
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
        let _ = ModifiedJulianDate::from_date(date);
    }

    /// Verifies that construction of a MJD based on a Gregorian date never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let _ = ModifiedJulianDate::from_date(date);
    }
}
