//! Time scales are used to relate time points with each other. They are, in effect, a manner to
//! link instances in time to a number of elapsed seconds since some epoch. The manner in which
//! this relation is established differs per time scale.

use num::Zero;

use crate::{
    DateTimeError, FineDateTimeError, FromDate,
    arithmetic::{
        FromUnit, Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute, TimeRepresentation,
        TryFromExact, Unit,
    },
    duration::{Duration, Hours, Minutes, Seconds},
    time_point::TimePoint,
};

mod glonasst;
pub use glonasst::*;
mod bdt;
pub use bdt::*;
mod gst;
pub use gst::*;
mod gpst;
pub use gpst::*;
mod local;
pub use local::*;
mod qzsst;
pub use qzsst::*;
mod tai;
pub use tai::*;
mod tcg;
pub use tcg::*;
mod tt;
pub use tt::*;
mod unix;
pub use unix::*;
mod utc;
pub use utc::*;

/// A time scale is a specification for measuring time. In this implementation, we specify this by
/// relating times to an elapsed duration since some reference epoch.
///
/// Additionally, all `TimeScale`s shall be able to convert `TimePoint`s from their scale to TAI
/// and the other way around. This is used to fundamentally connect all clocks.
pub trait TimeScale: Sized {
    /// The native `Period` in which a time scale's time points are expressed. This is the minimum
    /// unit needed to represent both its TAI and local time epochs. For most time scales, this is
    /// simply a day.
    type NativePeriod: Unit;

    /// The native `Representation` in which a time scale's time points are expressed. This is the
    /// minimum representation needed to be able to represent all possible `Date` values in the
    /// `NativePeriod` unit.
    type NativeRepresentation: TimeRepresentation;

    /// Returns the epoch of this time scale but expressed in TAI. This is useful for performing
    /// conversions between different time scales.
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod>;

    /// Returns the epoch of a time scale, expressed as a `LocalTime` in its own time scale. The
    /// result may be expressed in any type `T`, as long as this type can be constructed from some
    /// primitive. This function is allowed to panic if the epoch, expressed as `LocalDays`, cannot
    /// be represented by a value of type `T`.
    fn epoch_local() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod>;

    /// Returns whether this time scales incorporates leap seconds, i.e., whether the underlying
    /// "seconds since epoch" count also increases one second when a leap second is inserted.
    fn counts_leap_seconds() -> bool {
        false
    }

    /// Creates a `TimePoint` from some previously created `LocalDays` instance by adding a given
    /// time-of-day to it.
    fn from_local_datetime(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, Self::NativeRepresentation, Self::NativePeriod>, DateTimeError>
    where
        Self::NativePeriod: FromUnit<SecondsPerDay, Self::NativeRepresentation>
            + FromUnit<SecondsPerHour, Self::NativeRepresentation>
            + FromUnit<SecondsPerMinute, Self::NativeRepresentation>
            + FromUnit<Second, Self::NativeRepresentation>,
        Second: FromUnit<SecondsPerDay, Self::NativeRepresentation>
            + FromUnit<SecondsPerHour, Self::NativeRepresentation>
            + FromUnit<SecondsPerMinute, Self::NativeRepresentation>
            + FromUnit<Second, Self::NativeRepresentation>,
    {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(DateTimeError::InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }

        let hours = Hours::new(hour).try_cast().unwrap();
        let minutes = Minutes::new(minute).try_cast().unwrap();
        let seconds = Seconds::new(second).try_cast().unwrap();
        let epoch = Self::epoch_local();
        let local_time: LocalTime<Self::NativeRepresentation> =
            date.try_cast().unwrap().into_unit()
                + hours.into_unit()
                + minutes.into_unit()
                + seconds;
        let time_since_epoch = local_time.into_unit() - epoch;
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }

    /// Creates a `TimePoint` from some previously created `LocalDays` instance by adding a given
    /// time-of-day and subsecond fraction to it.
    fn from_subsecond_local_datetime<Representation, Period>(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration<Representation, Period>,
    ) -> Result<TimePoint<Self, Representation, Period>, FineDateTimeError<Representation, Period>>
    where
        Representation: TimeRepresentation + TryFromExact<Self::NativeRepresentation>,
        Period: Unit + FromDate<Representation> + FromUnit<Self::NativePeriod, Representation>,
        Second: FromDate<Self::NativeRepresentation>,
        Self::NativePeriod: FromDate<Self::NativeRepresentation>,
    {
        // We check that the number of subseconds does not exceed one second.
        let one = Seconds::new(Representation::one()).into_unit();
        let zero = Duration::zero();
        if subseconds < zero || subseconds >= one {
            return Err(FineDateTimeError::InvalidSubseconds { subseconds });
        }

        let seconds = Self::from_local_datetime(date, hour, minute, second)?
            .try_cast::<Representation>()
            .unwrap()
            .into_unit();
        Ok(seconds + subseconds)
    }
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another.
pub trait FromTimeScale<From: TimeScale>: TimeScale {
    /// Converts from a `TimePoint` in the `Self` `TimeScale` to an equivalent `TimePoint` in the
    /// `To` `TimeScale`. Note that the representations shall be the same between both
    /// `TimeScales`. Due to some time scale conversions being inexact relations (e.g., TAI to
    /// TDB), this may mean that some rounding is allowed to occur. Hence, it is advisable to
    /// upcast the `from` time point towards a higher-accuracy representation before converting.
    ///
    /// This function is allowed to panic for scenarios where the underlying `Representation`
    /// cannot represent the difference between two `TimePoint` reference epochs to a given
    /// `Period` resolution. For all other choices of `Representation` and `Period`, this
    /// conversion must be infallible.
    ///
    /// A default implementation of this function is provided that is valid for any two time scales
    /// that have the same time tick rate but that differ in epoch. This means that this
    /// implementation is valid, for example, for the TAI, UTC, Unix, and GPS clocks. It will not
    /// be valid for dynamic clocks.
    fn from_time_scale<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> TimePoint<Self, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation
            + TryFromExact<From::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
        Period: FromUnit<From::NativePeriod, From::NativeRepresentation>,
        Period: FromUnit<Self::NativePeriod, Self::NativeRepresentation>,
    {
        let time_since_from_epoch = from.elapsed_time_since_epoch();
        let from_epoch = From::epoch_tai()
            .into_unit::<Period>()
            .try_cast::<Representation>()
            .unwrap();
        let to_epoch = Self::epoch_tai()
            .into_unit::<Period>()
            .try_cast::<Representation>()
            .unwrap();
        // Note that this operation first rounds and then casts the epoch differences into the
        // proper units and representation. The representation cast may fail, if the difference in
        // epochs is not representable by the chosen representation (e.g., a `u8` cannot store the
        // number of seconds between the `To` and `From` epoch). In such cases, this conversion will
        // panic.
        // We check which epoch is latest in time, and flip the signs based on that. This is needed
        // so that we don't overflow (to below 0) when working with unsigned counts.
        if to_epoch > from_epoch {
            let epoch_difference: Duration<Representation, Period> = to_epoch - from_epoch;
            TimePoint::<Self, Representation, Period>::from_time_since_epoch(
                time_since_from_epoch - epoch_difference,
            )
        } else {
            let epoch_difference: Duration<Representation, Period> = from_epoch - to_epoch;
            TimePoint::<Self, Representation, Period>::from_time_since_epoch(
                time_since_from_epoch + epoch_difference,
            )
        }
    }
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another.
pub trait IntoTimeScale<Into: TimeScale>: TimeScale {
    /// Converts from a `TimePoint` in the `Self` `TimeScale` to an equivalent `TimePoint` in the
    /// `To` `TimeScale`. Note that the representations shall be the same between both
    /// `TimeScales`. Due to some time scale conversions being inexact relations (e.g., TAI to
    /// TDB), this may mean that some rounding is allowed to occur. Hence, it is advisable to
    /// upcast the `from` time point towards a higher-accuracy representation before converting.
    ///
    /// This function is allowed to panic for scenarios where the underlying `Representation`
    /// cannot represent the difference between two `TimePoint` reference epochs to a given
    /// `Period` resolution. For all other choices of `Representation` and `Period`, this
    /// conversion must be infallible.
    ///
    /// A default implementation of this function is provided that is valid for any two time scales
    /// that have the same time tick rate but that differ in epoch. This means that this
    /// implementation is valid, for example, for the TAI, UTC, Unix, and GPS clocks. It will not
    /// be valid for dynamic clocks.
    fn into_time_scale<Representation, Period>(
        from: TimePoint<Self, Representation, Period>,
    ) -> TimePoint<Into, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation
            + TryFromExact<Self::NativeRepresentation>
            + TryFromExact<Into::NativeRepresentation>,
        Period: FromUnit<Self::NativePeriod, Self::NativeRepresentation>,
        Period: FromUnit<Into::NativePeriod, Into::NativeRepresentation>;
}

impl<From: TimeScale, Into: TimeScale> IntoTimeScale<Into> for From
where
    Into: FromTimeScale<From>,
{
    fn into_time_scale<Representation, Period>(
        from: TimePoint<Self, Representation, Period>,
    ) -> TimePoint<Into, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation
            + TryFromExact<Self::NativeRepresentation>
            + TryFromExact<Into::NativeRepresentation>,
        Period: FromUnit<Self::NativePeriod, Self::NativeRepresentation>,
        Period: FromUnit<Into::NativePeriod, Into::NativeRepresentation>,
    {
        Into::from_time_scale(from)
    }
}

impl<T: TimeScale> FromTimeScale<T> for T {
    /// Conversion from a clock to itself is always possible and a no-op.
    fn from_time_scale<Representation, Period>(
        from: TimePoint<T, Representation, Period>,
    ) -> TimePoint<T, Representation, Period>
    where
        Representation: TimeRepresentation,
        Period: Unit,
    {
        from
    }
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another, though it is
/// allowed for this operation to fail. This is the case when applying leap seconds, for example:
/// the result may then be ambiguous or undefined, based on folds and gaps in time.
///
/// Similar to `TryFrom` and `TryInto`, it is advised to implement only `TryFromTimeScale`. The
/// equivalent `TryIntoTimeScale` trait implementation may then be derived.
pub trait TryFromTimeScale<From: TimeScale>: TimeScale {
    type Error: core::fmt::Debug;

    /// Tries to convert from one time scale to another. If this is not unambiguously possible,
    /// returns an error indicating why it is not.
    fn try_from_time_scale<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> Result<TimePoint<Self, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<From::NativePeriod, From::NativeRepresentation>
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<From::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>;
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another, though it is
/// allowed for this operation to fail. This is the case when applying leap seconds, for example:
/// the result may then be ambiguous or undefined, based on folds and gaps in time.
pub trait TryIntoTimeScale<Into: TimeScale>: TimeScale {
    type Error: core::fmt::Debug;

    /// Tries to convert from one time scale to another. If this is not unambiguously possible,
    /// returns an error indicating why it is not.
    fn try_into_time_scale<Representation, Period>(
        from: TimePoint<Self, Representation, Period>,
    ) -> Result<TimePoint<Into, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Into::NativePeriod, Into::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<Self::NativeRepresentation>
            + TryFromExact<Into::NativeRepresentation>;
}

impl<From: TimeScale, Into: TimeScale> TryFromTimeScale<From> for Into
where
    Into: FromTimeScale<From>,
{
    type Error = core::convert::Infallible;

    /// Default implementation of a "try" conversion whenever two time scales can already be
    /// converted infallibly.
    fn try_from_time_scale<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> Result<TimePoint<Into, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<From::NativePeriod, From::NativeRepresentation>
            + FromUnit<Into::NativePeriod, Into::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<From::NativeRepresentation>
            + TryFromExact<Into::NativeRepresentation>,
    {
        Ok(Self::from_time_scale(from))
    }
}

impl<From: TimeScale, Into: TimeScale> TryIntoTimeScale<Into> for From
where
    Into: TryFromTimeScale<From>,
{
    type Error = <Into as TryFromTimeScale<From>>::Error;

    fn try_into_time_scale<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> Result<TimePoint<Into, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<From::NativePeriod, From::NativeRepresentation>
            + FromUnit<Into::NativePeriod, Into::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<From::NativeRepresentation>
            + TryFromExact<Into::NativeRepresentation>,
    {
        Into::try_from_time_scale(from)
    }
}
