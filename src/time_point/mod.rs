//! A time point represents a duration that has passed since the epoch of some time scale. This is
//! similar to the C++ definition used in the `<chrono>` library.

use core::{
    hash::Hash,
    ops::{Add, AddAssign, Sub},
};

use num::Integer;

use crate::{
    DateTimeError, FineDateTimeError, IntoTimeScale, TryIntoTimeScale,
    arithmetic::{
        IntoUnit, Nano, Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute,
        TimeRepresentation, TryFromExact, TryIntoExact, Unit,
    },
    duration::Duration,
    time_scale::{LocalDays, TimeScale},
};

#[cfg(feature = "std")]
use crate::{TryFromTimeScale, Unix, UnixTime, arithmetic::FromUnit};

/// A time point indicates an elapsed duration with respect to the epoch of some time scale. It may
/// utilize arbitrary units and arbitrary precision, defined by the underlying `Representation` and
/// `Period`.
#[derive(Debug)]
pub struct TimePoint<TimeScale, Representation, Period = Second>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    duration: Duration<Representation, Period>,
    time_scale: core::marker::PhantomData<TimeScale>,
}

impl<Scale> TimePoint<Scale, u128, Nano> {
    /// Creates a `TimePoint` that represents the current epoch, as approximated by this machine's
    /// system time. Returns a timestamp in nanosecond resolution, but does not need to be (and
    /// most certainly will not be) accurate to nanoseconds.
    ///
    /// An error may be returned if the conversion from Unix time is ambiguous or undefined. This
    /// may happen in particular around leap seconds, where the conversion from Unix time to
    /// continuous time scales like TAI or UTC is impossible.
    #[cfg(feature = "std")]
    pub fn now() -> Result<Self, <Scale as TryFromTimeScale<Unix>>::Error>
    where
        Scale: TimeScale + TryFromTimeScale<Unix>,
        Nano: FromUnit<<Unix as TimeScale>::NativePeriod, u128>
            + FromUnit<<Scale as TimeScale>::NativePeriod, u128>,
    {
        let system_time = std::time::SystemTime::now();
        let unix_time = UnixTime::from(system_time);
        Scale::try_from_time_scale(unix_time)
    }
}

/// Tests that it is possible to obtain the current time without crashing. We do not actually check
/// for the resulting values, because we cannot make any guarantees about the time that we run
/// these tests at: they may return errors when run during leap seconds.
#[cfg(feature = "std")]
#[test]
fn get_current_time() {
    use crate::{GpsTime, TaiTime, TtTime, UnixTime, UtcTime};
    let _ = UnixTime::now().unwrap();
    let _ = GpsTime::now();
    let _ = TaiTime::now();
    let _ = TtTime::now();
    let _ = UtcTime::now();
}

impl<Scale, Representation, Period> TimePoint<Scale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    /// Creates a `TimePoint` directly from the given elapsed time since some `TimeScale` epoch.
    pub const fn from_time_since_epoch(duration: Duration<Representation, Period>) -> Self {
        Self {
            duration,
            time_scale: core::marker::PhantomData,
        }
    }

    /// Returns the elapsed time since the `TimeScale` epoch.
    pub fn elapsed_time_since_epoch(&self) -> Duration<Representation, Period>
    where
        Duration<Representation, Period>: Clone,
    {
        self.duration.clone()
    }

    /// Creates a `TimePoint` from some datetime and time-of-day, plus some additional subseconds.
    pub fn from_subsecond_datetime(
        date: impl Into<LocalDays<i64>>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<Self, FineDateTimeError<Representation, Period>>
    where
        Scale: TimeScale,
        Period: Unit,
        SecondsPerDay: IntoUnit<Period, Representation> + IntoUnit<Scale::NativePeriod, i64>,
        SecondsPerHour: IntoUnit<Period, Representation> + IntoUnit<Scale::NativePeriod, i64>,
        SecondsPerMinute: IntoUnit<Period, Representation> + IntoUnit<Scale::NativePeriod, i64>,
        Second: IntoUnit<Period, Representation> + IntoUnit<Scale::NativePeriod, i64>,
        Scale::NativePeriod: IntoUnit<Period, Representation>,
    {
        Scale::from_subsecond_local_datetime(date.into(), hour, minute, second, subseconds)
    }

    /// Converts towards a different time unit, rounding towards the nearest whole unit.
    pub fn round<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Period: Unit,
        Target: Unit,
    {
        TimePoint {
            duration: self.duration.round(),
            time_scale: core::marker::PhantomData,
        }
    }

    /// Converts towards a different time unit, rounding towards positive infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn ceil<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Period: Unit,
        Target: Unit,
    {
        TimePoint {
            duration: self.duration.ceil(),
            time_scale: core::marker::PhantomData,
        }
    }

    /// Converts towards a different time unit, rounding towards negative infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn floor<Target>(self) -> TimePoint<Scale, Representation, Target>
    where
        Period: Unit,
        Target: Unit,
    {
        TimePoint {
            duration: self.duration.floor(),
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<Scale> TimePoint<Scale, i64, <Scale as TimeScale>::NativePeriod>
where
    Scale: TimeScale,
{
    /// Creates a `TimePoint` from a historic date and an associated time-of-day.
    pub fn from_datetime(
        date: impl Into<LocalDays<i64>>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, DateTimeError>
    where
        Scale: TimeScale,
        SecondsPerDay: IntoUnit<Second, i64> + IntoUnit<Scale::NativePeriod, i64>,
        SecondsPerHour: IntoUnit<Second, i64> + IntoUnit<Scale::NativePeriod, i64>,
        SecondsPerMinute: IntoUnit<Second, i64> + IntoUnit<Scale::NativePeriod, i64>,
        Second: IntoUnit<Scale::NativePeriod, i64>,
    {
        Scale::from_local_datetime(date.into(), hour, minute, second)
    }
}

impl<Scale, Representation, Period: Unit> TimePoint<Scale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    /// Converts a `TimePoint` towards a different time unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `TimePoint` is a float.
    pub fn into_unit<Target: Unit>(self) -> TimePoint<Scale, Representation, Target>
    where
        Period: IntoUnit<Target, Representation>,
    {
        TimePoint {
            duration: self.duration.into_unit(),
            time_scale: core::marker::PhantomData,
        }
    }

    /// Tries to convert a `TimePoint` towards a different time unit. Only applies to integers (as
    /// all floats may be converted infallibly anyway). Will only return a result if the conversion
    /// is lossless.
    pub fn try_into_unit<Target: Unit>(self) -> Option<TimePoint<Scale, Representation, Target>>
    where
        Representation: Integer + TryFromExact<i128>,
    {
        Some(TimePoint {
            duration: self.duration.try_into_unit()?,
            time_scale: core::marker::PhantomData,
        })
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> TimePoint<Scale, Target, Period>
    where
        Representation: Into<Target>,
        Target: TimeRepresentation,
        Period: Unit,
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
        Representation: TryIntoExact<Target>,
        Target: TimeRepresentation,
        Period: Unit,
    {
        Some(TimePoint {
            duration: self.duration.try_cast()?,
            time_scale: core::marker::PhantomData,
        })
    }

    /// Transforms a time point towards another time scale.
    pub fn into_time_scale<Target>(self) -> TimePoint<Target, Representation, Period>
    where
        Target: TimeScale,
        Scale: TimeScale + IntoTimeScale<Target>,
        Period: FromUnit<Scale::NativePeriod, Representation>
            + FromUnit<Target::NativePeriod, Representation>,
    {
        <Scale as IntoTimeScale<Target>>::into_time_scale(self)
    }

    /// Tries to transform a time point into another time scale.
    #[allow(clippy::type_complexity)]
    pub fn try_into_time_scale<Target>(
        self,
    ) -> Result<TimePoint<Target, Representation, Period>, <Scale as TryIntoTimeScale<Target>>::Error>
    where
        Scale: TimeScale,
        Target: TryFromTimeScale<Scale> + TimeScale,
        Period: FromUnit<Scale::NativePeriod, Representation>
            + FromUnit<Target::NativePeriod, Representation>,
    {
        Target::try_from_time_scale(self)
    }
}

impl<TimeScale, Representation, Period> Copy for TimePoint<TimeScale, Representation, Period>
where
    Representation: Copy + TimeRepresentation,
    Period: Unit,
{
}

impl<TimeScale, Representation, Period> Clone for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
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
    Representation: TimeRepresentation + PartialEq,
    Period: Unit,
{
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
    }
}

impl<TimeScale, Representation, Period> Eq for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation + Eq,
    Period: Unit,
{
}

impl<TimeScale, Representation, Period> PartialOrd for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation + PartialOrd,
    Period: Unit,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}

impl<TimeScale, Representation, Period> Ord for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation + Ord,
    Period: Unit,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.duration.cmp(&other.duration)
    }
}

impl<TimeScale, Representation, Period> Hash for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation + Hash,
    Period: Unit,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.duration.hash(state);
    }
}

impl<TimeScale, Representation, Period: Unit> Sub for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    type Output = Duration<Representation, Period>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.duration - rhs.duration
    }
}

impl<TimeScale, Representation, Period: Unit> Sub<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    type Output = TimePoint<TimeScale, Representation, Period>;

    fn sub(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Self {
            duration: self.duration - rhs,
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<TimeScale, Representation, Period: Unit> Add<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    type Output = TimePoint<TimeScale, Representation, Period>;

    fn add(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Self {
            duration: self.duration + rhs,
            time_scale: core::marker::PhantomData,
        }
    }
}

impl<TimeScale, Representation, Period: Unit> AddAssign<Duration<Representation, Period>>
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation + AddAssign<Representation>,
    Period: Unit,
{
    fn add_assign(&mut self, rhs: Duration<Representation, Period>) {
        self.duration += rhs;
    }
}

#[cfg(kani)]
impl<TimeScale, Representation: kani::Arbitrary, Period> kani::Arbitrary
    for TimePoint<TimeScale, Representation, Period>
where
    Representation: TimeRepresentation,
    Period: Unit,
{
    fn any() -> Self {
        TimePoint {
            duration: kani::any(),
            time_scale: core::marker::PhantomData,
        }
    }
}
