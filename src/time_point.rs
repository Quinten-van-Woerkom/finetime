//! Definition of the `TimePoint` type (and associated types and methods), which implements the
//! fundamental timekeeping logic of this library.

use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_traits::{Bounded, Zero};

use crate::{
    Convert, Date, DateTime, DateTimeRepresentation, Duration, Fraction, GregorianDate,
    HistoricDate, JulianDate, Month, MulCeil, MulFloor, MulRound, TryConvert, UnitRatio,
    errors::{InvalidGregorianDateTime, InvalidHistoricDateTime, InvalidJulianDateTime},
    fractional_digits::FractionalDigits,
    units::Second,
};

/// A `TimePoint` identifies a specific instant in time. It is templated on a `Representation` and
/// `Period`, which the define the characteristics of the `Duration` type used to represent the
/// time elapsed since the epoch of the underlying time scale `Scale`.
pub struct TimePoint<Scale: ?Sized, Representation, Period = Second> {
    time_since_epoch: Duration<Representation, Period>,
    time_scale: core::marker::PhantomData<Scale>,
}

impl<Scale: ?Sized, Representation, Period> TimePoint<Scale, Representation, Period> {
    /// Constructs a new `TimePoint` from a known time since epoch.
    pub const fn from_time_since_epoch(time_since_epoch: Duration<Representation, Period>) -> Self {
        Self {
            time_since_epoch,
            time_scale: core::marker::PhantomData,
        }
    }

    /// Returns the time elapsed since the epoch of the time scale associated with this instant.
    pub const fn time_since_epoch(&self) -> Duration<Representation, Period>
    where
        Representation: Copy,
    {
        self.time_since_epoch
    }

    /// Converts this `TimePoint` into a different unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `TimePoint` is a float.
    pub fn into_unit<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Representation: Convert<Period, Target>,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.into_unit())
    }

    /// Tries to convert a `TimePoint` towards a different time unit. Will only return a result if
    /// the conversion is lossless.
    pub fn try_into_unit<Target>(self) -> Option<TimePoint<Scale, Representation, Target>>
    where
        Representation: TryConvert<Period, Target>,
    {
        Some(TimePoint::from_time_since_epoch(
            self.time_since_epoch.try_into_unit()?,
        ))
    }

    /// Converts towards a different time unit, rounding towards the nearest whole unit.
    pub fn round<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Representation: MulRound<Fraction, Output = Representation>,
        Period: UnitRatio,
        Target: UnitRatio,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.round())
    }

    /// Converts towards a different time unit, rounding towards positive infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn ceil<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Representation: MulCeil<Fraction, Output = Representation>,
        Period: UnitRatio,
        Target: UnitRatio,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.ceil())
    }

    /// Converts towards a different time unit, rounding towards negative infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn floor<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Representation: MulFloor<Fraction, Output = Representation>,
        Period: UnitRatio,
        Target: UnitRatio,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.floor())
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> TimePoint<Scale, Target, Period>
    where
        Representation: Into<Target>,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.cast())
    }

    /// Converts towards a different representation. If the underlying representation cannot store
    /// the result of this cast, returns an appropriate `Error`.
    pub fn try_cast<Target>(
        self,
    ) -> Result<TimePoint<Scale, Target, Period>, <Representation as TryInto<Target>>::Error>
    where
        Representation: TryInto<Target>,
    {
        Ok(TimePoint::from_time_since_epoch(
            self.time_since_epoch.try_cast()?,
        ))
    }
}

impl<Scale> TimePoint<Scale, i64, Second>
where
    Scale: ?Sized + DateTime,
{
    /// Constructs a `TimePoint` in the given time scale based on the date and time-of-day.
    pub fn from_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, Scale::Error> {
        Scale::time_point_from_datetime(date, hour, minute, second)
    }

    /// Constructs a `TimePoint` in the given time scale, based on a historic date-time.
    pub fn from_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidHistoricDateTime<Scale::Error>> {
        let date = Date::from_historic_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidHistoricDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a Gregorian date-time.
    pub fn from_gregorian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidGregorianDateTime<Scale::Error>> {
        let date = Date::from_gregorian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidGregorianDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a Julian date-time.
    pub fn from_julian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidJulianDateTime<Scale::Error>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }

    /// Maps a `TimePoint` towards the corresponding date and time-of-day.
    pub fn to_datetime(&self) -> (Date<i32>, u8, u8, u8) {
        Scale::datetime_from_time_point(*self)
    }

    /// Maps a `TimePoint` towards the corresponding historic date and time-of-day.
    pub fn to_historic_datetime(&self) -> (HistoricDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.to_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding proleptic Gregorian date and time-of-day.
    pub fn to_gregorian_datetime(&self) -> (GregorianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.to_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding Julian date and time-of-day.
    pub fn to_julian_datetime(&self) -> (JulianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.to_datetime();
        (date.into(), hour, minute, second)
    }
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized + DateTime,
    Representation: DateTimeRepresentation + Convert<Second, Period>,
{
    /// Constructs a `TimePoint` from a given date and subsecond-accuracy time.
    pub fn from_fine_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, Scale::Error> {
        Scale::time_point_from_fine_datetime(date, hour, minute, second, subseconds)
    }

    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy historic
    /// date-time.
    pub fn from_fine_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, InvalidHistoricDateTime<Scale::Error>> {
        let date = Date::from_historic_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidHistoricDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy Gregorian
    /// date-time.
    pub fn from_fine_gregorian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, InvalidGregorianDateTime<Scale::Error>> {
        let date = Date::from_gregorian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidGregorianDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy Julian
    /// date-time.
    pub fn from_fine_julian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, InvalidJulianDateTime<Scale::Error>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized + DateTime,
    Representation: DateTimeRepresentation + Convert<Second, Period>,
    Period: UnitRatio,
{
    pub fn to_fine_datetime(&self) -> (Date<i32>, u8, u8, u8, Duration<Representation, Period>) {
        Scale::fine_datetime_from_time_point(*self)
    }

    pub fn to_fine_historic_datetime(
        &self,
    ) -> (HistoricDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.to_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    pub fn to_fine_gregorian_datetime(
        &self,
    ) -> (GregorianDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.to_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    pub fn to_fine_julian_datetime(
        &self,
    ) -> (JulianDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.to_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }
}

impl<Scale, Representation, Period> Display for TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized + DateTime,
    Representation: DateTimeRepresentation + Convert<Second, Period> + Zero + FractionalDigits,
    Period: UnitRatio,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (historic_date, hour, minute, second, subseconds) = self.to_fine_historic_datetime();
        write!(
            f,
            "{:04}-{:02}-{:02}T{hour:02}:{minute:02}:{second:02}",
            historic_date.year(),
            historic_date.month() as u8,
            historic_date.day(),
        )?;

        if !subseconds.is_zero() {
            write!(f, ".")?;

            // Set maximum number of digits after the decimal point printed based on precision
            // argument given to the formatter.
            let max_digits_printed = f.precision().unwrap_or(9);

            for digit in subseconds.fractional_digits(max_digits_printed) {
                write!(f, "{digit}")?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn check_formatting_i64(
    string: &str,
    year: i32,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    milliseconds: i64,
) {
    let time = crate::TaiTime::from_fine_historic_datetime(
        year,
        month,
        day,
        hour,
        minute,
        second,
        crate::MilliSeconds::new(milliseconds),
    )
    .unwrap();
    assert_eq!(time.to_string(), string);
}

/// Verifies formatting for some known values.
#[test]
fn formatting_i64() {
    use crate::Month::*;
    check_formatting_i64("1958-01-01T00:00:00.001", 1958, January, 1, 0, 0, 0, 1);
    check_formatting_i64("1958-01-02T00:00:00", 1958, January, 2, 0, 0, 0, 0);
    check_formatting_i64("1960-01-01T12:34:56.789", 1960, January, 1, 12, 34, 56, 789);
    check_formatting_i64("1961-01-01T00:00:00", 1961, January, 1, 0, 0, 0, 0);
    check_formatting_i64("1970-01-01T00:00:00", 1970, January, 1, 0, 0, 0, 0);
    check_formatting_i64("1976-01-01T23:59:59.999", 1976, January, 1, 23, 59, 59, 999);
    check_formatting_i64("2025-07-16T16:23:24", 2025, July, 16, 16, 23, 24, 0);
    check_formatting_i64("2034-12-26T08:02:37.123", 2034, December, 26, 8, 2, 37, 123);
    check_formatting_i64("2760-04-01T21:59:58", 2760, April, 1, 21, 59, 58, 0);
    check_formatting_i64("1643-01-04T01:01:33", 1643, January, 4, 1, 1, 33, 0);
}

/// Verifies that truncation is properly applied when the underlying fraction exceeds the number of
/// digits specified in the formatting precision (or 9 by default, if none is specified).
#[test]
fn truncated_format() {
    let time = crate::TaiTime::from_fine_historic_datetime(
        1998,
        Month::December,
        17,
        23,
        21,
        58,
        crate::PicoSeconds::new(450103789401i128),
    )
    .unwrap();
    assert_eq!(time.to_string(), "1998-12-17T23:21:58.450103789");
}

/// Verifies that formatting does not panic for a large randomized range of values.
#[test]
fn random_formatting() {
    use crate::TaiTime;
    use core::str::FromStr;
    use rand::prelude::*;
    let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(76);
    for _ in 0..10_000 {
        let ticks_since_epoch = rng.random::<i64>();
        let time_since_epoch = crate::NanoSeconds::new(ticks_since_epoch);
        let time = TaiTime::from_time_since_epoch(time_since_epoch);
        let string = format!("{time:.9}");
        let time2 = TaiTime::from_str(string.as_str()).unwrap();
        assert_eq!(time, time2);
    }
}

#[cfg(kani)]
impl<Scale, Representation: kani::Arbitrary, Period> kani::Arbitrary
    for TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized,
{
    fn any() -> Self {
        TimePoint::from_time_since_epoch(kani::any())
    }
}

impl<Scale, Representation, Period> Debug for TimePoint<Scale, Representation, Period>
where
    Representation: Debug,
    Scale: ?Sized,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimePoint")
            .field("time_since_epoch", &self.time_since_epoch)
            .field("time_scale", &self.time_scale)
            .finish()
    }
}

impl<Scale, Representation, Period> Copy for TimePoint<Scale, Representation, Period>
where
    Representation: Copy,
    Scale: ?Sized,
{
}

impl<Scale, Representation, Period> Clone for TimePoint<Scale, Representation, Period>
where
    Representation: Clone,
    Scale: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_time_since_epoch(self.time_since_epoch.clone())
    }
}

impl<Scale, Representation, Period> PartialEq for TimePoint<Scale, Representation, Period>
where
    Representation: PartialEq,
    Scale: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.time_since_epoch == other.time_since_epoch
    }
}

impl<Scale, Representation, Period> Eq for TimePoint<Scale, Representation, Period>
where
    Representation: Eq,
    Scale: ?Sized,
{
}

impl<Scale, Representation, Period> PartialOrd for TimePoint<Scale, Representation, Period>
where
    Representation: PartialOrd,
    Scale: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.time_since_epoch.partial_cmp(&other.time_since_epoch)
    }
}

impl<Scale, Representation, Period> Ord for TimePoint<Scale, Representation, Period>
where
    Representation: Ord,
    Scale: ?Sized,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.time_since_epoch.cmp(&other.time_since_epoch)
    }
}

impl<Scale, Representation, Period> Hash for TimePoint<Scale, Representation, Period>
where
    Representation: Hash,
    Scale: ?Sized,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.time_since_epoch.hash(state);
    }
}

impl<Scale, R1, R2, Period> Sub<TimePoint<Scale, R2, Period>> for TimePoint<Scale, R1, Period>
where
    R1: Sub<R2>,
    Scale: ?Sized,
{
    type Output = Duration<<R1 as Sub<R2>>::Output, Period>;

    fn sub(self, rhs: TimePoint<Scale, R2, Period>) -> Self::Output {
        self.time_since_epoch - rhs.time_since_epoch
    }
}

impl<Scale, R1, R2, Period> Add<Duration<R2, Period>> for TimePoint<Scale, R1, Period>
where
    R1: Add<R2>,
    Scale: ?Sized,
{
    type Output = TimePoint<Scale, <R1 as Add<R2>>::Output, Period>;

    fn add(self, rhs: Duration<R2, Period>) -> Self::Output {
        TimePoint::from_time_since_epoch(self.time_since_epoch + rhs)
    }
}

impl<Scale, R1, R2, Period> AddAssign<Duration<R2, Period>> for TimePoint<Scale, R1, Period>
where
    R1: AddAssign<R2>,
    Scale: ?Sized,
{
    fn add_assign(&mut self, rhs: Duration<R2, Period>) {
        self.time_since_epoch += rhs;
    }
}

impl<Scale, R1, R2, Period> Sub<Duration<R2, Period>> for TimePoint<Scale, R1, Period>
where
    R1: Sub<R2>,
    Scale: ?Sized,
{
    type Output = TimePoint<Scale, <R1 as Sub<R2>>::Output, Period>;

    fn sub(self, rhs: Duration<R2, Period>) -> Self::Output {
        TimePoint::from_time_since_epoch(self.time_since_epoch - rhs)
    }
}

impl<Scale, R1, R2, Period> SubAssign<Duration<R2, Period>> for TimePoint<Scale, R1, Period>
where
    R1: SubAssign<R2>,
    Scale: ?Sized,
{
    fn sub_assign(&mut self, rhs: Duration<R2, Period>) {
        self.time_since_epoch -= rhs;
    }
}

impl<Scale, Representation, Period> Bounded for TimePoint<Scale, Representation, Period>
where
    Representation: Bounded,
    Scale: ?Sized,
{
    fn min_value() -> Self {
        Self::from_time_since_epoch(Duration::<Representation, Period>::min_value())
    }

    fn max_value() -> Self {
        Self::from_time_since_epoch(Duration::<Representation, Period>::max_value())
    }
}
