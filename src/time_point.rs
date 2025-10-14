//! Definition of the `TimePoint` type (and associated types and methods), which implements the
//! fundamental timekeeping logic of this library.

use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_traits::{Bounded, Zero};

use crate::{
    ConvertUnit, Date, Duration, Fraction, FractionalDigits, FromDateTime, FromFineDateTime,
    GregorianDate, HalfDays, HistoricDate, IntoDateTime, IntoFineDateTime, JulianDate, JulianDay,
    ModifiedJulianDate, Month, MulCeil, MulFloor, MulRound, TryConvertUnit, TryFromExact,
    TryIntoExact, UnitRatio,
    errors::{InvalidGregorianDateTime, InvalidHistoricDateTime, InvalidJulianDateTime},
    time_scale::{TimeScale, UniformDateTimeScale},
    units::{Second, SecondsPerDay, SecondsPerHalfDay},
};

/// A `TimePoint` identifies a specific instant in time. It is templated on a `Representation` and
/// `Period`, which the define the characteristics of the `Duration` type used to represent the
/// time elapsed since the epoch of the underlying time scale `Scale`.
pub struct TimePoint<Scale: ?Sized, Representation = i64, Period: ?Sized = Second> {
    time_since_epoch: Duration<Representation, Period>,
    time_scale: core::marker::PhantomData<Scale>,
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period> {
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

    /// Returns the raw underlying representation of this time point.
    pub const fn count(&self) -> Representation
    where
        Representation: Copy,
    {
        self.time_since_epoch().count()
    }

    /// Converts this `TimePoint` into a different unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `TimePoint` is a float.
    pub fn into_unit<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Representation: ConvertUnit<Period, Target>,
        Target: ?Sized,
    {
        TimePoint::from_time_since_epoch(self.time_since_epoch.into_unit())
    }

    /// Tries to convert a `TimePoint` towards a different time unit. Will only return a result if
    /// the conversion is lossless.
    pub fn try_into_unit<Target>(self) -> Option<TimePoint<Scale, Representation, Target>>
    where
        Representation: TryConvertUnit<Period, Target>,
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
    ) -> Result<TimePoint<Scale, Target, Period>, <Representation as TryIntoExact<Target>>::Error>
    where
        Representation: TryIntoExact<Target>,
    {
        Ok(TimePoint::from_time_since_epoch(
            self.time_since_epoch.try_cast()?,
        ))
    }
}

impl<Scale: ?Sized> TimePoint<Scale, i64, Second>
where
    Self: FromDateTime,
{
    /// Constructs a `TimePoint` in the given time scale, based on a historic date-time.
    pub fn from_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidHistoricDateTime<<Self as FromDateTime>::Error>> {
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
    ) -> Result<Self, InvalidGregorianDateTime<<Self as FromDateTime>::Error>> {
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
    ) -> Result<Self, InvalidJulianDateTime<<Self as FromDateTime>::Error>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period>
where
    Scale: UniformDateTimeScale,
    Representation: Copy
        + Sub<Output = Representation>
        + TryFromExact<i32>
        + ConvertUnit<SecondsPerHalfDay, Period>,
{
    /// Constructs a time point from a Julian day, expressed in the resulting time scale itself.
    /// A Julian day is the count of days since noon on 1 January 4713 BC (in the proleptic Julian
    /// calendar).
    ///
    /// Conversions from Julian days into `TimePoint`s are supported only for uniform date time
    /// scales. For non-uniform time scales, leap second days result in ambiguous and difficult to
    /// implement interpretations of the fractional part of a day. Based on the "Resolution B1 on
    /// the use of Julian Dates" of the IAU, it is also not recommended to use such Julian date
    /// expressions: hence, we do not support it.
    pub fn from_julian_day(jd: JulianDay<Representation, Period>) -> Self {
        const JULIAN_EPOCH: Date<i32> = match Date::from_julian_date(-4712, Month::January, 1) {
            Ok(epoch) => epoch,
            Err(_) => panic!("Internal error: start of Julian period found invalid"),
        };
        let epoch_julian_day = Scale::EPOCH
            .elapsed_calendar_days_since(JULIAN_EPOCH)
            .into_unit()
            - HalfDays::new(1i32);
        let time_since_epoch = jd.time_since_epoch()
            - epoch_julian_day
                .try_cast()
                .unwrap_or_else(|_| panic!())
                .into_unit();
        Self::from_time_since_epoch(time_since_epoch)
    }
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period>
where
    Scale: UniformDateTimeScale,
    Representation: Copy
        + Sub<Output = Representation>
        + TryFromExact<i32>
        + ConvertUnit<SecondsPerDay, Period>,
{
    /// Constructs a time point from a modified Julian date, expressed in the resulting time scale
    /// itself. The modified Julian date uses 17 November, 1858 (historic calendar) as epoch, or
    /// 2400000.5 days less than the Julian day.
    ///
    /// Conversions from modified Julian days into `TimePoint`s are supported only for uniform date
    /// time scales. For non-uniform time scales, leap second days result in ambiguous and
    /// difficult to implement interpretations of the fractional part of a day. Based on the
    /// "Resolution B1 on the use of Julian Dates" of the IAU, it is also not recommended to use
    /// such Julian date expressions: hence, we do not support it.
    pub fn from_modified_julian_date(mjd: ModifiedJulianDate<Representation, Period>) -> Self {
        const MODIFIED_JULIAN_EPOCH: Date<i32> =
            match Date::from_historic_date(1858, Month::November, 17) {
                Ok(epoch) => epoch,
                Err(_) => panic!("Internal error: start of modified Julian period found invalid"),
            };
        let epoch_julian_day = Scale::EPOCH.elapsed_calendar_days_since(MODIFIED_JULIAN_EPOCH);
        let time_since_epoch = mjd.time_since_epoch()
            - epoch_julian_day
                .try_cast()
                .unwrap_or_else(|_| panic!())
                .into_unit();
        Self::from_time_since_epoch(time_since_epoch)
    }
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period>
where
    Scale: TimeScale,
    Representation: Copy
        + Add<Output = Representation>
        + TryFromExact<i32>
        + ConvertUnit<SecondsPerHalfDay, Period>,
{
    /// Converts this time point into the equivalent Julian day representation.
    pub fn into_julian_day(&self) -> JulianDay<Representation, Period> {
        const JULIAN_EPOCH: Date<i32> = match Date::from_julian_date(-4712, Month::January, 1) {
            Ok(epoch) => epoch,
            Err(_) => panic!("Internal error: start of Julian period found invalid"),
        };
        let epoch_julian_day = Scale::EPOCH
            .elapsed_calendar_days_since(JULIAN_EPOCH)
            .into_unit()
            - HalfDays::new(1);
        let time_since_epoch = epoch_julian_day
            .try_cast()
            .unwrap_or_else(|_| panic!())
            .into_unit()
            + self.time_since_epoch();
        JulianDay::from_time_since_epoch(time_since_epoch)
    }
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period>
where
    Scale: TimeScale,
    Representation: Copy
        + Add<Output = Representation>
        + TryFromExact<i32>
        + ConvertUnit<SecondsPerDay, Period>,
{
    /// Converts this time point into the equivalent Julian day representation.
    pub fn into_modified_julian_date(&self) -> ModifiedJulianDate<Representation, Period> {
        const MODIFIED_JULIAN_EPOCH: Date<i32> =
            match Date::from_historic_date(1858, Month::November, 17) {
                Ok(epoch) => epoch,
                Err(_) => panic!("Internal error: start of modified Julian period found invalid"),
            };
        let epoch_julian_day = Scale::EPOCH.elapsed_calendar_days_since(MODIFIED_JULIAN_EPOCH);
        let time_since_epoch = epoch_julian_day
            .try_cast()
            .unwrap_or_else(|_| panic!())
            .into_unit()
            + self.time_since_epoch();
        ModifiedJulianDate::from_time_since_epoch(time_since_epoch)
    }
}

#[cfg(test)]
fn check_julian_date(year: i32, month: Month, day: u8) {
    use crate::TtTime;

    let julian_day = JulianDay::from_historic_date(year, month, day).unwrap();
    let modified_julian_date = ModifiedJulianDate::from_historic_date(year, month, day).unwrap();
    let time = TtTime::from_julian_day(julian_day);
    let time2 = TtTime::from_modified_julian_date(modified_julian_date);
    let time3 = TtTime::from_historic_datetime(year, month, day, 0, 0, 0).unwrap();
    assert_eq!(time.cast().into_unit(), time3);
    assert_eq!(time.into_julian_day(), julian_day);
    assert_eq!(time2.cast().into_unit(), time3);
    assert_eq!(time2.into_modified_julian_date(), modified_julian_date);
}

#[test]
fn julian_dates() {
    use crate::Month;
    check_julian_date(2000, Month::January, 1);
    check_julian_date(1999, Month::January, 1);
    check_julian_date(1987, Month::January, 27);
    check_julian_date(1987, Month::June, 19);
    check_julian_date(1988, Month::June, 27);
    check_julian_date(1988, Month::July, 19);
    check_julian_date(1900, Month::January, 1);
    check_julian_date(1600, Month::January, 1);
    check_julian_date(1600, Month::December, 31);
    check_julian_date(837, Month::April, 10);
    check_julian_date(-123, Month::December, 31);
    check_julian_date(-122, Month::January, 1);
    check_julian_date(-1000, Month::July, 12);
    check_julian_date(-1000, Month::February, 29);
    check_julian_date(-1001, Month::August, 17);
    check_julian_date(-4712, Month::January, 1);
}

impl<Scale: ?Sized, Representation> TimePoint<Scale, Representation, Second>
where
    Self: IntoDateTime,
{
    /// Maps a `TimePoint` towards the corresponding historic date and time-of-day.
    pub fn into_historic_datetime(self) -> (HistoricDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding proleptic Gregorian date and time-of-day.
    pub fn into_gregorian_datetime(self) -> (GregorianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding Julian date and time-of-day.
    pub fn into_julian_datetime(self) -> (JulianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }
}

impl<Scale, Representation, Period> FromFineDateTime<Representation, Period>
    for TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized,
    Period: ?Sized,
    Representation: Add<Representation, Output = Representation>
        + ConvertUnit<Second, Period>
        + TryFromExact<i64>,
    TimePoint<Scale, i64, Second>: FromDateTime,
{
    type Error = <TimePoint<Scale, i64, Second> as FromDateTime>::Error;

    fn from_fine_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, Self::Error> {
        let coarse_time_point: TimePoint<Scale, Representation, Second> =
            TimePoint::from_datetime(date, hour, minute, second)?
                .try_into_exact()
                .unwrap_or_else(|_| panic!());
        Ok(coarse_time_point.into_unit() + subseconds)
    }
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period>
where
    Self: FromFineDateTime<Representation, Period>,
    TimePoint<Scale, i64, Second>: FromDateTime,
    Scale: ?Sized,
    Period: ?Sized,
{
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
    ) -> Result<
        Self,
        InvalidHistoricDateTime<<Self as FromFineDateTime<Representation, Period>>::Error>,
    > {
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
    ) -> Result<
        Self,
        InvalidGregorianDateTime<<Self as FromFineDateTime<Representation, Period>>::Error>,
    > {
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
    ) -> Result<
        Self,
        InvalidJulianDateTime<<Self as FromFineDateTime<Representation, Period>>::Error>,
    > {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale, Representation, Period> IntoFineDateTime<Representation, Period>
    for TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized,
    Representation: Copy
        + ConvertUnit<Second, Period>
        + MulFloor<Fraction, Output = Representation>
        + Sub<Representation, Output = Representation>,
    Period: UnitRatio + ?Sized,
    TimePoint<Scale, Representation, Second>: IntoDateTime,
{
    fn into_fine_datetime(self) -> (Date<i32>, u8, u8, u8, Duration<Representation, Period>) {
        let coarse_time_point = self.floor::<Second>();
        let subseconds = self - coarse_time_point.into_unit::<Period>();
        let (date, hour, minute, second) = coarse_time_point.into_datetime();
        (date, hour, minute, second, subseconds)
    }
}

impl<Scale: ?Sized, Representation, Period: ?Sized> TimePoint<Scale, Representation, Period>
where
    Self: IntoFineDateTime<Representation, Period>,
{
    pub fn into_fine_historic_datetime(
        self,
    ) -> (HistoricDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    pub fn into_fine_gregorian_datetime(
        self,
    ) -> (GregorianDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    pub fn into_fine_julian_datetime(
        self,
    ) -> (JulianDate, u8, u8, u8, Duration<Representation, Period>) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }
}

impl<Scale, Representation, Period> Display for TimePoint<Scale, Representation, Period>
where
    Self: IntoFineDateTime<Representation, Period>,
    Scale: ?Sized + TimeScale,
    Duration<Representation, Period>: Zero,
    Representation: Copy + FractionalDigits,
    Period: UnitRatio + ?Sized,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (historic_date, hour, minute, second, subseconds) = self.into_fine_historic_datetime();
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
            let max_digits_printed = f.precision();
            for digit in subseconds.decimal_digits(max_digits_printed) {
                write!(f, "{digit}")?;
            }
        }

        write!(f, " {}", Scale::ABBREVIATION)
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
    check_formatting_i64("1958-01-01T00:00:00.001 TAI", 1958, January, 1, 0, 0, 0, 1);
    check_formatting_i64("1958-01-02T00:00:00 TAI", 1958, January, 2, 0, 0, 0, 0);
    check_formatting_i64(
        "1960-01-01T12:34:56.789 TAI",
        1960,
        January,
        1,
        12,
        34,
        56,
        789,
    );
    check_formatting_i64("1961-01-01T00:00:00 TAI", 1961, January, 1, 0, 0, 0, 0);
    check_formatting_i64("1970-01-01T00:00:00 TAI", 1970, January, 1, 0, 0, 0, 0);
    check_formatting_i64(
        "1976-01-01T23:59:59.999 TAI",
        1976,
        January,
        1,
        23,
        59,
        59,
        999,
    );
    check_formatting_i64("2025-07-16T16:23:24 TAI", 2025, July, 16, 16, 23, 24, 0);
    check_formatting_i64(
        "2034-12-26T08:02:37.123 TAI",
        2034,
        December,
        26,
        8,
        2,
        37,
        123,
    );
    check_formatting_i64("2760-04-01T21:59:58 TAI", 2760, April, 1, 21, 59, 58, 0);
    check_formatting_i64("1643-01-04T01:01:33 TAI", 1643, January, 4, 1, 1, 33, 0);
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn check_formatting_f64(
    string: &str,
    year: i32,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    milliseconds: f64,
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
fn formatting_f64() {
    use crate::Month::*;
    check_formatting_f64("1958-01-01T00:00:00.001 TAI", 1958, January, 1, 0, 0, 0, 1.);
    check_formatting_f64("1958-01-02T00:00:00 TAI", 1958, January, 2, 0, 0, 0, 0.);
    check_formatting_f64(
        "1960-01-01T12:34:56.789 TAI",
        1960,
        January,
        1,
        12,
        34,
        56,
        789.,
    );
    check_formatting_f64("1961-01-01T00:00:00 TAI", 1961, January, 1, 0, 0, 0, 0.);
    check_formatting_f64("1970-01-01T00:00:00 TAI", 1970, January, 1, 0, 0, 0, 0.);
    check_formatting_f64(
        "1976-01-01T23:59:59.999 TAI",
        1976,
        January,
        1,
        23,
        59,
        59,
        999.,
    );
    check_formatting_f64("2025-07-16T16:23:24 TAI", 2025, July, 16, 16, 23, 24, 0.);
    check_formatting_f64(
        "2034-12-26T08:02:37.123 TAI",
        2034,
        December,
        26,
        8,
        2,
        37,
        123.,
    );
    check_formatting_f64("2760-04-01T21:59:58 TAI", 2760, April, 1, 21, 59, 58, 0.);
    check_formatting_f64("1643-01-04T01:01:33 TAI", 1643, January, 4, 1, 1, 33, 0.);
}

/// Verifies that truncation is properly applied when the underlying fraction exceeds the number of
/// digits specified in the formatting precision (or 9 by default, if none is specified).
#[test]
fn truncated_format() {
    let time = crate::UtcTime::from_fine_historic_datetime(
        1998,
        Month::December,
        17,
        23,
        21,
        58,
        crate::PicoSeconds::new(450103789401i128),
    )
    .unwrap();
    assert_eq!(format!("{time:.9}"), "1998-12-17T23:21:58.450103789 UTC");
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
    Period: ?Sized,
{
    fn any() -> Self {
        TimePoint::from_time_since_epoch(kani::any())
    }
}

impl<Scale, Representation, Period> Debug for TimePoint<Scale, Representation, Period>
where
    Representation: Debug,
    Scale: ?Sized,
    Period: ?Sized,
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
    Period: ?Sized,
{
}

impl<Scale, Representation, Period> Clone for TimePoint<Scale, Representation, Period>
where
    Representation: Clone,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn clone(&self) -> Self {
        Self::from_time_since_epoch(self.time_since_epoch.clone())
    }
}

impl<Scale, Representation, Period> PartialEq for TimePoint<Scale, Representation, Period>
where
    Representation: PartialEq,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.time_since_epoch == other.time_since_epoch
    }
}

impl<Scale, Representation, Period> Eq for TimePoint<Scale, Representation, Period>
where
    Representation: Eq,
    Scale: ?Sized,
    Period: ?Sized,
{
}

impl<Scale, Representation, Period> PartialOrd for TimePoint<Scale, Representation, Period>
where
    Representation: PartialOrd,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.time_since_epoch.partial_cmp(&other.time_since_epoch)
    }
}

impl<Scale, Representation, Period> Ord for TimePoint<Scale, Representation, Period>
where
    Representation: Ord,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.time_since_epoch.cmp(&other.time_since_epoch)
    }
}

impl<Scale, Representation, Period> Hash for TimePoint<Scale, Representation, Period>
where
    Representation: Hash,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.time_since_epoch.hash(state);
    }
}

impl<Scale, Representation, Period> Sub for TimePoint<Scale, Representation, Period>
where
    Duration<Representation, Period>: Sub<Output = Duration<Representation, Period>>,
    Scale: ?Sized,
    Period: ?Sized,
{
    type Output = Duration<Representation, Period>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.time_since_epoch - rhs.time_since_epoch
    }
}

impl<Scale, Representation, Period> Add<Duration<Representation, Period>>
    for TimePoint<Scale, Representation, Period>
where
    Duration<Representation, Period>: Add<Output = Duration<Representation, Period>>,
    Scale: ?Sized,
    Period: ?Sized,
{
    type Output = Self;

    fn add(self, rhs: Duration<Representation, Period>) -> Self::Output {
        TimePoint::from_time_since_epoch(self.time_since_epoch + rhs)
    }
}

impl<Scale, Representation, Period> AddAssign<Duration<Representation, Period>>
    for TimePoint<Scale, Representation, Period>
where
    Duration<Representation, Period>: AddAssign,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn add_assign(&mut self, rhs: Duration<Representation, Period>) {
        self.time_since_epoch += rhs;
    }
}

impl<Scale, Representation, Period> Sub<Duration<Representation, Period>>
    for TimePoint<Scale, Representation, Period>
where
    Duration<Representation, Period>: Sub<Output = Duration<Representation, Period>>,
    Scale: ?Sized,
    Period: ?Sized,
{
    type Output = Self;

    fn sub(self, rhs: Duration<Representation, Period>) -> Self::Output {
        TimePoint::from_time_since_epoch(self.time_since_epoch - rhs)
    }
}

impl<Scale, Representation, Period> SubAssign<Duration<Representation, Period>>
    for TimePoint<Scale, Representation, Period>
where
    Duration<Representation, Period>: SubAssign,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn sub_assign(&mut self, rhs: Duration<Representation, Period>) {
        self.time_since_epoch -= rhs;
    }
}

impl<Scale, Representation, Period> Bounded for TimePoint<Scale, Representation, Period>
where
    Representation: Bounded,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn min_value() -> Self {
        Self::from_time_since_epoch(Duration::<Representation, Period>::min_value())
    }

    fn max_value() -> Self {
        Self::from_time_since_epoch(Duration::<Representation, Period>::max_value())
    }
}

impl<Scale, R1, R2, Period> TryFromExact<TimePoint<Scale, R2, Period>>
    for TimePoint<Scale, R1, Period>
where
    R1: TryFromExact<R2>,
    Scale: ?Sized,
    Period: ?Sized,
{
    type Error = <R1 as TryFromExact<R2>>::Error;

    fn try_from_exact(value: TimePoint<Scale, R2, Period>) -> Result<Self, Self::Error> {
        let time_since_epoch = value.time_since_epoch.try_into_exact()?;
        Ok(Self::from_time_since_epoch(time_since_epoch))
    }
}
