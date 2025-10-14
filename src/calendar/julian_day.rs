//! Representation of some calendrical time point as the elapsed number of (potentially fractional)
//! days since the start of the Julian period.

use core::ops::{Add, Sub};

use crate::{
    ConvertUnit, Date, Days, Duration, HalfDays, Month, TryIntoExact,
    errors::{InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate},
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JulianDay<Representation, Period: ?Sized = SecondsPerHalfDay> {
    time_since_epoch: Duration<Representation, Period>,
}

/// The Julian date of the Unix epoch is useful as constant in some calculations.
const JULIAN_DAY_UNIX_EPOCH: HalfDays<i32> = HalfDays::new(4881175);

impl<Representation> JulianDay<Representation, SecondsPerDay> {
    /// Convenience function that constructs a Julian day directly from some day count.
    pub const fn new(jd: Representation) -> Self {
        Self::from_time_since_epoch(Days::new(jd))
    }
}

impl<Representation, Period: ?Sized> JulianDay<Representation, Period> {
    /// Constructs a Julian day from some duration since the epoch of the Julian period.
    pub const fn from_time_since_epoch(time_since_epoch: Duration<Representation, Period>) -> Self {
        Self { time_since_epoch }
    }

    /// Returns the time elapsed since the epoch of the Julian period.
    pub const fn time_since_epoch(&self) -> Duration<Representation, Period>
    where
        Representation: Copy,
    {
        self.time_since_epoch
    }

    /// Constructs a Julian day from some given calendar date.
    pub fn from_date(date: Date<Representation>) -> Self
    where
        Representation: Copy
            + From<i32>
            + Add<Representation, Output = Representation>
            + ConvertUnit<SecondsPerDay, Period>
            + ConvertUnit<SecondsPerHalfDay, Period>,
    {
        Self {
            time_since_epoch: date.time_since_epoch().into_unit()
                + JULIAN_DAY_UNIX_EPOCH.cast().into_unit(),
        }
    }

    pub fn into_date(&self) -> Date<Representation>
    where
        Representation: Copy
            + From<i32>
            + Sub<Representation, Output = Representation>
            + ConvertUnit<Period, SecondsPerDay>
            + ConvertUnit<SecondsPerHalfDay, SecondsPerDay>,
    {
        Date::from_time_since_epoch(
            self.time_since_epoch.into_unit() - JULIAN_DAY_UNIX_EPOCH.cast().into_unit(),
        )
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> JulianDay<Target, Period>
    where
        Duration<Representation, Period>: Into<Duration<Target, Period>>,
    {
        JulianDay::from_time_since_epoch(self.time_since_epoch.into())
    }

    /// Converts towards a different representation. If the underlying representation cannot store
    /// the result of this cast, returns `None`.
    pub fn try_cast<Target>(
        self,
    ) -> Result<JulianDay<Target, Period>, <Representation as TryIntoExact<Target>>::Error>
    where
        Representation: TryIntoExact<Target>,
    {
        Ok(JulianDay::from_time_since_epoch(
            self.time_since_epoch.try_cast()?,
        ))
    }
}

impl JulianDay<i32, SecondsPerHalfDay> {
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

impl<Representation, Period: ?Sized> From<JulianDay<Representation, Period>>
    for Date<Representation>
where
    Representation: Copy
        + From<i32>
        + Sub<Representation, Output = Representation>
        + ConvertUnit<Period, SecondsPerDay>
        + ConvertUnit<SecondsPerHalfDay, SecondsPerDay>,
{
    fn from(value: JulianDay<Representation, Period>) -> Self {
        value.into_date()
    }
}

impl<Representation, Period: ?Sized> From<Date<Representation>>
    for JulianDay<Representation, Period>
where
    Representation: Copy
        + From<i32>
        + Add<Representation, Output = Representation>
        + ConvertUnit<SecondsPerDay, Period>
        + ConvertUnit<SecondsPerHalfDay, Period>,
{
    fn from(value: Date<Representation>) -> Self {
        Self::from_date(value)
    }
}

/// Verifies this implementation by computing the `JulianDay` for some known (computed manually or
/// obtained elsewhere) time stamp. If it doesn't match the given `time_since_epoch`, panics.
#[cfg(test)]
fn check_historic_julian_day(year: i32, month: Month, day: u8, time_since_epoch: HalfDays<i32>) {
    assert_eq!(
        JulianDay::from_historic_date(year, month, day)
            .unwrap()
            .time_since_epoch(),
        time_since_epoch,
    );
}

/// Compares some computed JD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::Month::*;
    check_historic_julian_day(2000, January, 1, HalfDays::new(4903089));
    check_historic_julian_day(1999, January, 1, HalfDays::new(4902359));
    check_historic_julian_day(1987, January, 27, HalfDays::new(4893645));
    check_historic_julian_day(1987, June, 19, HalfDays::new(4893931));
    check_historic_julian_day(1988, January, 27, HalfDays::new(4894375));
    check_historic_julian_day(1988, June, 19, HalfDays::new(4894663));
    check_historic_julian_day(1900, January, 1, HalfDays::new(4830041));
    check_historic_julian_day(1600, January, 1, HalfDays::new(4610895));
    check_historic_julian_day(1600, December, 31, HalfDays::new(4611625));
    check_historic_julian_day(837, April, 10, HalfDays::new(4053743));
    check_historic_julian_day(-123, December, 31, HalfDays::new(3352993));
    check_historic_julian_day(-122, January, 1, HalfDays::new(3352995));
    check_historic_julian_day(-1000, July, 12, HalfDays::new(2712001));
    check_historic_julian_day(-1000, February, 29, HalfDays::new(2711733));
    check_historic_julian_day(-1001, August, 17, HalfDays::new(2711341));
    check_historic_julian_day(-4712, January, 1, -HalfDays::new(1));
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a MJD based on a date never panics.
    #[kani::proof]
    fn from_date_never_panics() {
        let date: Date<i32> = kani::any();
        kani::assume(
            date.time_since_epoch().count() <= i32::MAX / 2 - JULIAN_DAY_UNIX_EPOCH.count(),
        );
        kani::assume(date.time_since_epoch().count() >= i32::MIN / 2);
        let _ = JulianDay::<i32>::from_date(date);
    }
}
