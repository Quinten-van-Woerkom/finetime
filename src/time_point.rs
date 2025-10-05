//! Definition of the `TimePoint` type (and associated types and methods), which implements the
//! fundamental timekeeping logic of this library.

use core::{
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_traits::Bounded;

use crate::{
    Convert, Date, DateTime, Duration, Fraction, Month, MulCeil, MulFloor, MulRound, TryConvert,
    UnitRatio,
    errors::{InvalidGregorianDateTime, InvalidHistoricDateTime, InvalidJulianDateTime},
    units::Second,
};

/// A `TimePoint` identifies a specific instant in time. It is templated on a `Representation` and
/// `Period`, which the define the characteristics of the `Duration` type used to represent the
/// time elapsed since the epoch of the underlying time scale `Scale`.
#[derive(Debug)]
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
    ) -> Result<Self, Scale::UnrepresentableDateTime> {
        let time_seconds = Scale::time_point_from_datetime(date, hour, minute, second)?;
        let time = time_seconds
            .try_cast()
            .unwrap_or_else(|_| panic!())
            .into_unit();
        Ok(time)
    }

    /// Maps a `TimePoint` towards the corresponding date and time-of-day.
    pub fn to_datetime(&self) -> Result<(Date<i32>, u8, u8, u8), Scale::UnrepresentableTimePoint> {
        Scale::datetime_from_time_point(*self)
    }

    /// Constructs a `TimePoint` in the given time scale, based on a historic date-time.
    pub fn from_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidHistoricDateTime<Scale::UnrepresentableDateTime>> {
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
    ) -> Result<Self, InvalidGregorianDateTime<Scale::UnrepresentableDateTime>> {
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
    ) -> Result<Self, InvalidJulianDateTime<Scale::UnrepresentableDateTime>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period>
where
    Scale: ?Sized + DateTime,
    Representation:
        From<i64> + Convert<Second, Period> + Add<Representation, Output = Representation>,
{
    /// Constructs a `TimePoint` from a given date and subsecond-accuracy time.
    pub fn from_fine_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, Scale::UnrepresentableDateTime> {
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
    ) -> Result<Self, InvalidHistoricDateTime<Scale::UnrepresentableDateTime>> {
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
    ) -> Result<Self, InvalidGregorianDateTime<Scale::UnrepresentableDateTime>> {
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
    ) -> Result<Self, InvalidJulianDateTime<Scale::UnrepresentableDateTime>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

#[cfg(kani)]
impl<Scale, Representation: kani::Arbitrary, Period> kani::Arbitrary
    for TimePoint<Scale, Representation, Period>
{
    fn any() -> Self {
        TimePoint::from_time_since_epoch(kani::any())
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
