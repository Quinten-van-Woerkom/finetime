//! Representation of some calendrical time point as the elapsed number of (potentially fractional)
//! days since the start of the Julian period.

use core::ops::{Add, Div, Mul, Sub};

use num::NumCast;

use crate::{
    Days, Duration, Hours, LocalDays, LocalTime,
    units::{IsValidConversion, Ratio, SecondsPerDay, SecondsPerHour},
};

/// Representation of calendrical dates in terms of Julian Days (JD). A Julian date is the number
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct JulianDate<T, Period = SecondsPerDay> {
    day: Duration<T, Period>,
}

impl<T, Period> JulianDate<T, Period> {
    /// Constructs a new Julian date directly from some duration since the start of the Julian
    /// period.
    pub const fn new(duration: Duration<T, Period>) -> Self {
        Self { day: duration }
    }
}

impl JulianDate<i64, SecondsPerHour> {
    /// Constructs a Julian date from some given calendar date.
    pub fn from_date(date: impl Into<LocalDays<i64>>) -> Self {
        let local_days: LocalTime<i64, SecondsPerDay> = date.into();
        let local_hours: LocalTime<i64, SecondsPerHour> = local_days.convert();
        Self::from(local_hours)
    }
}

impl<Representation, Period> From<LocalTime<Representation, Period>>
    for JulianDate<Representation, Period>
where
    Period: Ratio,
    Representation: Copy
        + From<i64>
        + Add<Representation, Output = Representation>
        + Mul<Representation, Output = Representation>
        + Div<Representation, Output = Representation>
        + NumCast,
    (): IsValidConversion<Representation, SecondsPerDay, Period>,
    (): IsValidConversion<Representation, SecondsPerHour, Period>,
{
    /// Transforming from `LocalDays` (since Unix epoch) to the equivalent `JulianDate` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: LocalTime<Representation, Period>) -> Self {
        Self {
            day: value.elapsed_time_since_epoch()
                + Days::new(2440587).cast().convert()
                + Hours::new(12).cast().convert(),
        }
    }
}

impl<Representation, Period> From<JulianDate<Representation, Period>>
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
    (): IsValidConversion<Representation, SecondsPerHour, Period>,
{
    /// Transforming to `LocalDays` (since Unix epoch) from the equivalent `JulianDate` is
    /// nothing more than a constant offset of the number of days between the two epochs.
    fn from(value: JulianDate<Representation, Period>) -> Self {
        Self::from_time_since_epoch(
            value.day - Days::new(2440587).cast().convert() - Hours::new(12).cast().convert(),
        )
    }
}

/// Compares some computed JD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::calendar::Date;
    use crate::calendar::Month::*;
    assert_eq!(
        JulianDate::from_date(Date::new(2000, January, 1).unwrap()),
        JulianDate::new(Days::new(2451544).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1999, January, 1).unwrap()),
        JulianDate::new(Days::new(2451179).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1987, January, 27).unwrap()),
        JulianDate::new(Days::new(2446822).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1987, June, 19).unwrap()),
        JulianDate::new(Days::new(2446965).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1988, January, 27).unwrap()),
        JulianDate::new(Days::new(2447187).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1988, June, 19).unwrap()),
        JulianDate::new(Days::new(2447331).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1900, January, 1).unwrap()),
        JulianDate::new(Days::new(2415020).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1600, January, 1).unwrap()),
        JulianDate::new(Days::new(2305447).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(1600, December, 31).unwrap()),
        JulianDate::new(Days::new(2305812).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(837, April, 10).unwrap()),
        JulianDate::new(Days::new(2026871).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-123, December, 31).unwrap()),
        JulianDate::new(Days::new(1676496).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-122, January, 1).unwrap()),
        JulianDate::new(Days::new(1676497).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-1000, July, 12).unwrap()),
        JulianDate::new(Days::new(1356000).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-1000, February, 29).unwrap()),
        JulianDate::new(Days::new(1355866).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-1001, August, 17).unwrap()),
        JulianDate::new(Days::new(1355670).convert() + Hours::new(12i64))
    );
    assert_eq!(
        JulianDate::from_date(Date::new(-4712, January, 1).unwrap()),
        JulianDate::new(-Hours::new(12))
    );
}

impl<Representation, Period> Sub for JulianDate<Representation, Period>
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
    for JulianDate<Representation, Period>
where
    Representation: Add<Output = Representation>,
    Period: Ratio,
{
    type Output = JulianDate<Representation, Period>;

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
        let _ = JulianDate::from_date(date);
    }

    /// Verifies that construction of a MJD based on a Gregorian date never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let _ = JulianDate::from_date(date);
    }
}
