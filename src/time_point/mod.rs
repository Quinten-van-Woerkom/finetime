//! A time point represents a duration that has passed since the epoch of some time scale. This is
//! similar to the C++ definition used in the `<chrono>` library.

use core::{
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use num::{Bounded, Integer, NumCast, One, Zero, traits::NumOps};

use crate::{
    DateTimeError, FineDateTimeError, TimeScaleConversion,
    duration::Duration,
    time_scale::{LocalDays, TimeScale},
    units::{
        IsValidConversion, LiteralRatio, Ratio, SecondsPerDay, SecondsPerHour, SecondsPerMinute,
    },
};

/// A time point indicates an elapsed duration with respect to the epoch of some time scale. It may
/// utilize arbitrary units and arbitrary precision, defined by the underlying `Representation` and
/// `Period`.
#[derive(Debug)]
pub struct TimePoint<TimeScale, Representation, Period = LiteralRatio<1>> {
    duration: Duration<Representation, Period>,
    time_scale: core::marker::PhantomData<TimeScale>,
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period> {
    /// Creates a `TimePoint` directly from the given elapsed time since some `TimeScale` epoch.
    pub const fn from_time_since_epoch(duration: Duration<Representation, Period>) -> Self {
        Self {
            duration,
            time_scale: core::marker::PhantomData,
        }
    }

    /// Returns the elapsed time since the `TimeScale` epoch.
    pub const fn elapsed_time_since_epoch(&self) -> Duration<Representation, Period>
    where
        Duration<Representation, Period>: Copy,
    {
        self.duration
    }

    /// Creates a `TimePoint` from some datetime and time-of-day, plus some additional subseconds.
    pub fn from_subsecond_datetime(
        date: impl Into<LocalDays<Representation>>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, FineDateTimeError<Representation, Period>>
    where
        Scale: TimeScale,
        Period: Ratio,
        Representation: NumCast + NumOps + From<u8> + PartialOrd + Clone + One + Zero,
        (): IsValidConversion<Representation, SecondsPerDay, Period>
            + IsValidConversion<Representation, SecondsPerHour, Period>
            + IsValidConversion<Representation, SecondsPerMinute, Period>
            + IsValidConversion<Representation, LiteralRatio<1>, Period>
            + IsValidConversion<Representation, SecondsPerDay, LiteralRatio<1>>
            + IsValidConversion<Representation, SecondsPerHour, LiteralRatio<1>>
            + IsValidConversion<Representation, SecondsPerMinute, LiteralRatio<1>>,
    {
        Scale::from_subsecond_local_datetime(date.into(), hour, minute, second, subseconds)
    }
}

impl<Scale, Representation> TimePoint<Scale, Representation> {
    /// Creates a `TimePoint` from a historic date and an associated time-of-day.
    pub fn from_datetime(
        date: impl Into<LocalDays<Representation>>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, DateTimeError<Representation>>
    where
        Scale: TimeScale,
        Representation: NumCast + NumOps + From<u8> + Clone,
        (): IsValidConversion<Representation, SecondsPerDay, LiteralRatio<1>>
            + IsValidConversion<Representation, SecondsPerHour, LiteralRatio<1>>
            + IsValidConversion<Representation, SecondsPerMinute, LiteralRatio<1>>,
    {
        Scale::from_local_datetime(date.into(), hour, minute, second)
    }
}

impl<Scale, Representation, Period: Ratio> TimePoint<Scale, Representation, Period> {
    /// Converts a `TimePoint` towards a different time unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `TimePoint` is a float.
    pub fn convert<Target: Ratio>(self) -> TimePoint<Scale, Representation, Target>
    where
        (): IsValidConversion<Representation, Period, Target>,
        Representation: Mul<Representation, Output = Representation>
            + Div<Representation, Output = Representation>
            + NumCast,
    {
        TimePoint {
            duration: self.duration.convert(),
            time_scale: core::marker::PhantomData,
        }
    }

    /// Tries to convert a `TimePoint` towards a different time unit. Only applies to integers (as
    /// all floats may be converted infallibly anyway). Will only return a result if the conversion
    /// is lossless.
    pub fn try_convert<Target: Ratio>(self) -> Option<TimePoint<Scale, Representation, Target>>
    where
        Representation: NumCast + Integer + Bounded + Copy,
    {
        Some(TimePoint {
            duration: self.duration.try_convert()?,
            time_scale: core::marker::PhantomData,
        })
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> TimePoint<Scale, Target, Period>
    where
        Target: From<Representation>,
    {
        TimePoint {
            duration: self.duration.cast(),
            time_scale: core::marker::PhantomData,
        }
    }

    /// Converts towards a different representation. If the underlying representation cannot store
    /// the result of this cast, returns `None`.
    pub fn try_cast<Target>(self) -> Option<TimePoint<Scale, Target, Period>>
    where
        Representation: NumCast,
        Target: NumCast,
    {
        Some(TimePoint {
            duration: self.duration.try_cast()?,
            time_scale: core::marker::PhantomData,
        })
    }

    /// Transforms a time point towards another time scale.
    pub fn transform<Target>(self) -> TimePoint<Target, Representation, Period>
    where
        Target: TimeScale,
        Scale: TimeScale,
        Representation: Copy + NumCast + NumOps,
        (): TimeScaleConversion<Scale, Target>,
    {
        <() as TimeScaleConversion<Scale, Target>>::convert(self)
    }
}

impl<TimeScale, Representation, Period> Copy for TimePoint<TimeScale, Representation, Period> where
    Representation: Copy
{
}

impl<TimeScale, Representation, Period> Clone for TimePoint<TimeScale, Representation, Period>
where
    Representation: Clone,
{
    fn clone(&self) -> Self {
        Self {
            duration: self.duration.clone(),
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<TimeScale, Representation, Period> PartialEq for TimePoint<TimeScale, Representation, Period>
where
    Representation: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
    }
}

impl<TimeScale, Representation, Period> Eq for TimePoint<TimeScale, Representation, Period> where
    Representation: Eq
{
}

impl<TimeScale, Representation, Period> PartialOrd for TimePoint<TimeScale, Representation, Period>
where
    Representation: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}

impl<TimeScale, Representation, Period> Ord for TimePoint<TimeScale, Representation, Period>
where
    Representation: Ord,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.duration.cmp(&other.duration)
    }
}

impl<TimeScale, Representation, Period> Hash for TimePoint<TimeScale, Representation, Period>
where
    Representation: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
    }
}

impl<TimeScale, Representation, Period: Ratio> Sub for TimePoint<TimeScale, Representation, Period>
where
    Representation: Sub<Representation, Output = Representation>,
{
    type Output = Duration<Representation, Period>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.duration - rhs.duration
    }
}

impl<TimeScale, Representation, Period: Ratio> Sub<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: Sub<Representation, Output = Representation>,
{
    type Output = TimePoint<TimeScale, Representation, Period>;

    fn sub(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Self {
            duration: self.duration - rhs,
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<TimeScale, Representation, Period: Ratio> Add<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: Add<Representation, Output = Representation>,
{
    type Output = TimePoint<TimeScale, Representation, Period>;

    fn add(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Self {
            duration: self.duration + rhs,
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<TimeScale, Representation, Period: Ratio> AddAssign<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: AddAssign<Representation>,
{
    fn add_assign(&mut self, rhs: Duration<Representation, Period>) {
        self.duration += rhs;
    }
}

#[cfg(kani)]
impl<TimeScale, Representation: kani::Arbitrary, Period> kani::Arbitrary
    for TimePoint<TimeScale, Representation, Period>
{
    fn any() -> Self {
        TimePoint {
            duration: kani::any(),
            time_scale: core::marker::PhantomData,
        }
    }
}
