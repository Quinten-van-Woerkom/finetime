//! A time point represents a duration that has passed since the epoch of some time scale. This is
//! similar to the C++ definition used in the `<chrono>` library.

use core::{
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use num::{Bounded, Integer, NumCast};

use crate::{
    duration::{
        Duration,
        units::{IsValidConversion, LiteralRatio, Ratio},
    },
    time_scale::{TimeScale, local::LocalDays},
};

/// A time point indicates an elapsed duration with respect to the epoch of some time scale. It may
/// utilize arbitrary units and arbitrary precision, defined by the underlying `Representation` and
/// `Period`.
#[derive(Debug)]
#[repr(C)]
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
        Representation: NumCast
            + From<u8>
            + Sub<Representation, Output = Representation>
            + Add<Representation, Output = Representation>
            + Mul<Representation, Output = Representation>
            + Div<Representation, Output = Representation>
            + Clone,
        (): IsValidConversion<Representation, LiteralRatio<86400>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<3600>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<60>, LiteralRatio<1>>,
    {
        Scale::from_local_datetime(date.into(), hour, minute, second)
    }

    /// Creates a `TimePoint` from some previously created `LocalDays` instance by adding a given
    /// time-of-day to it.
    pub fn from_local_datetime(
        date: LocalDays<Representation>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, DateTimeError<Representation>>
    where
        Scale: TimeScale,
        Representation: NumCast
            + From<u8>
            + Sub<Representation, Output = Representation>
            + Add<Representation, Output = Representation>
            + Mul<Representation, Output = Representation>
            + Div<Representation, Output = Representation>
            + Clone,
        (): IsValidConversion<Representation, LiteralRatio<86400>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<3600>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<60>, LiteralRatio<1>>,
    {
        Scale::from_local_datetime(date, hour, minute, second)
    }
}

impl<TimeScale, Representation, Period: Ratio> TimePoint<TimeScale, Representation, Period> {
    /// Converts a `TimePoint` towards a different time unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `TimePoint` is a float.
    pub fn convert<Target: Ratio>(self) -> TimePoint<TimeScale, Representation, Target>
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
    pub fn try_convert<Target: Ratio>(self) -> Option<TimePoint<TimeScale, Representation, Target>>
    where
        Representation: NumCast + Integer + Bounded + Copy,
    {
        Some(TimePoint {
            duration: self.duration.try_convert()?,
            time_scale: core::marker::PhantomData,
        })
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> TimePoint<TimeScale, Target, Period>
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
    pub fn try_cast<Target>(self) -> Option<TimePoint<TimeScale, Target, Period>>
    where
        Representation: NumCast,
        Target: NumCast,
    {
        Some(TimePoint {
            duration: self.duration.try_cast()?,
            time_scale: core::marker::PhantomData,
        })
    }
}

/// Errors that may be returned when combining a calendar date with a time-of-day to create a
/// `TimePoint`. Includes errors that
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateTimeError<T> {
    /// Returned when the given time-of-day does not exist in general (independent of whether the
    /// used time scale has leap seconds).
    InvalidTimeOfDay { hour: u8, minute: u8, second: u8 },
    /// Returned when the requested datetime has a 61st second but is not actually situated at a
    /// leap second insertion.
    NoLeapSecondInsertion {
        date: LocalDays<T>,
        hour: u8,
        minute: u8,
        second: u8,
    },
    /// Returned when the requested datetime does not exist because of a leap second deletion.
    LeapSecondDeletion {
        date: LocalDays<T>,
        hour: u8,
        minute: u8,
        second: u8,
    },
    /// Returned when the requested datetime could not fit in a `TimePoint` with the given
    /// `Representation`.
    NotRepresentable {
        date: LocalDays<T>,
        hour: u8,
        minute: u8,
        second: u8,
    },
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
