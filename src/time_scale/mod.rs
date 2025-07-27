//! Time scales are used to relate time points with each other. They are, in effect, a manner to
//! link instances in time to a number of elapsed seconds since some epoch. The manner in which
//! this relation is established differs per time scale.

use num::{NumCast, One, Zero, traits::NumOps};

use crate::{
    DateTimeError, FineDateTimeError,
    duration::{Duration, Hours, Minutes, Seconds},
    time_point::TimePoint,
    units::{
        IsValidConversion, LiteralRatio, MulExact, Ratio, SecondsPerDay, SecondsPerHour,
        SecondsPerMinute,
    },
};

mod gpst;
pub use gpst::*;
mod local;
pub use local::*;
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
    type NativePeriod: Ratio;

    /// Returns the epoch of this time scale but expressed in TAI. This is useful for performing
    /// conversions between different time scales.
    fn epoch_tai<T>() -> TaiTime<T, Self::NativePeriod>
    where
        T: NumCast;

    /// Returns the epoch of a time scale, expressed as a `LocalTime` in its own time scale. The
    /// result may be expressed in any type `T`, as long as this type can be constructed from some
    /// primitive. This function is allowed to panic if the epoch, expressed as `LocalDays`, cannot
    /// be represented by a value of type `T`.
    fn epoch_local<T>() -> LocalTime<T, Self::NativePeriod>
    where
        T: NumCast;

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
    ) -> Result<TimePoint<Self, i64, Self::NativePeriod>, DateTimeError>
    where
        (): IsValidConversion<i64, SecondsPerDay, Self::NativePeriod>
            + IsValidConversion<i64, SecondsPerHour, Self::NativePeriod>
            + IsValidConversion<i64, SecondsPerMinute, Self::NativePeriod>
            + IsValidConversion<i64, LiteralRatio<1>, Self::NativePeriod>
            + IsValidConversion<i64, SecondsPerDay, LiteralRatio<1>>
            + IsValidConversion<i64, SecondsPerHour, LiteralRatio<1>>
            + IsValidConversion<i64, SecondsPerMinute, LiteralRatio<1>>,
    {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(DateTimeError::InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }

        let hours = Hours::new(hour).cast();
        let minutes = Minutes::new(minute).cast();
        let seconds = Seconds::new(second).cast();
        let epoch = Self::epoch_local();
        let local_time: LocalTime<i64> =
            date.convert() + hours.convert() + minutes.convert() + seconds;
        let time_since_epoch = local_time.convert() - epoch;
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
        Period: Ratio,
        Representation: NumCast + NumOps + From<u8> + PartialOrd + Clone + One + Zero,
        (): IsValidConversion<Representation, SecondsPerDay, Period>
            + IsValidConversion<Representation, SecondsPerHour, Period>
            + IsValidConversion<Representation, SecondsPerMinute, Period>
            + IsValidConversion<Representation, LiteralRatio<1>, Period>
            + IsValidConversion<i64, SecondsPerDay, Self::NativePeriod>
            + IsValidConversion<i64, SecondsPerHour, Self::NativePeriod>
            + IsValidConversion<i64, SecondsPerMinute, Self::NativePeriod>
            + IsValidConversion<i64, LiteralRatio<1>, Self::NativePeriod>
            + IsValidConversion<Representation, Self::NativePeriod, Period>,
    {
        // We check that the number of subseconds does not exceed one second.
        let one = Seconds::new(Representation::one()).convert();
        let zero = Duration::zero();
        if subseconds < zero || subseconds >= one {
            return Err(FineDateTimeError::InvalidSubseconds { subseconds });
        }

        let seconds = Self::from_local_datetime(date, hour, minute, second)?
            .try_cast()
            .unwrap();
        Ok(seconds.convert() + subseconds)
    }

    /// Creates a time point in this time scale based on a time point in TAI. Note that some
    /// rounding is permitted to occur here: not all time scales can be related exactly to TAI.
    fn from_tai<Representation, Period>(
        time_point: TimePoint<Tai, Representation, Period>,
    ) -> TimePoint<Self, Representation, Period>
    where
        (): TimeScaleConversion<Tai, Self>
            + IsValidConversion<i64, Self::NativePeriod, Period>
            + IsValidConversion<i64, <Tai as TimeScale>::NativePeriod, Period>,
        Period: Ratio,
        Representation: Copy + NumCast + NumOps + MulExact,
    {
        <() as TimeScaleConversion<Tai, Self>>::transform(time_point)
    }

    /// Creates a TAI time point based on a time point in this time scale. Rounding is permitted,
    /// as not all time scales can be exactly related to TAI.
    fn to_tai<Representation, Period>(
        time_point: TimePoint<Self, Representation, Period>,
    ) -> TimePoint<Tai, Representation, Period>
    where
        (): TimeScaleConversion<Self, Tai>
            + IsValidConversion<i64, Self::NativePeriod, Period>
            + IsValidConversion<i64, <Tai as TimeScale>::NativePeriod, Period>,
        Period: Ratio,
        Representation: Copy + NumCast + NumOps + MulExact,
    {
        <() as TimeScaleConversion<Self, Tai>>::transform(time_point)
    }
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another.
pub trait TimeScaleConversion<From: TimeScale, To: TimeScale> {
    /// Converts from a `TimePoint` in the `From` `TimeScale` to an equivalent `TimePoint` in the
    /// `To` `TimeScale`. Note that the representations shall be the same between both
    /// `TimeScales`. Due to some time scale conversions being inexact relations (e.g., TAI to
    /// TDB), this may mean that some is allowed to occur. Hence, it is advisable to upcast the
    /// `from` time point towards a higher-accuracy representation before converting.
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
    fn transform<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> TimePoint<To, Representation, Period>
    where
        Period: Ratio,
        Representation: Copy + NumCast + NumOps + MulExact,
        (): IsValidConversion<i64, <From as TimeScale>::NativePeriod, Period>
            + IsValidConversion<i64, <To as TimeScale>::NativePeriod, Period>,
    {
        let time_since_from_epoch = from.elapsed_time_since_epoch();
        let from_epoch = From::epoch_tai().convert();
        let to_epoch = To::epoch_tai().convert();
        // Note that this operation first rounds and then casts the epoch differences into the
        // proper units and representation. The representation cast may fail, if the difference in
        // epochs is not representable by the chosen representation (e.g., a `u8` cannot store the
        // number of seconds between the `To` and `From` epoch). In such cases, this conversion will
        // panic.
        // We check which epoch is latest in time, and flip the signs based on that. This is needed
        // so that we don't overflow (to below 0) when working with unsigned counts.
        if to_epoch > from_epoch {
            let epoch_difference: Duration<Representation, Period> =
                (to_epoch - from_epoch).round().try_cast().unwrap();
            TimePoint::<To, Representation, Period>::from_time_since_epoch(
                time_since_from_epoch - epoch_difference,
            )
        } else {
            let epoch_difference: Duration<Representation, Period> =
                (from_epoch - to_epoch).round().try_cast().unwrap();
            TimePoint::<To, Representation, Period>::from_time_since_epoch(
                time_since_from_epoch + epoch_difference,
            )
        }
    }
}

impl<T: TimeScale> TimeScaleConversion<T, T> for () {
    /// Conversion from a clock to itself is always possible and a no-op.
    fn transform<Representation, Period>(
        from: TimePoint<T, Representation, Period>,
    ) -> TimePoint<T, Representation, Period> {
        from
    }
}

/// Used to indicate that it is possible to convert from one `TimeScale` to another, though it is
/// allowed for this operation to fail. This is the case when applying leap seconds, for example:
/// the result may then be ambiguous or undefined, based on folds and gaps in time.
pub trait TryTimeScaleConversion<From: TimeScale, To: TimeScale, Representation, Period: Ratio> {
    type Error: core::fmt::Debug;

    /// Tries to convert from one time scale to another. If this is not unambiguously possible,
    /// returns an error indicating why it is not.
    fn try_convert(
        from: TimePoint<From, Representation, Period>,
    ) -> Result<TimePoint<To, Representation, Period>, Self::Error>
    where
        (): IsValidConversion<i64, <From as TimeScale>::NativePeriod, Period>
            + IsValidConversion<i64, <To as TimeScale>::NativePeriod, Period>;
}

impl<From: TimeScale, To: TimeScale, Representation, Period>
    TryTimeScaleConversion<From, To, Representation, Period> for ()
where
    (): TimeScaleConversion<From, To>,
    Period: Ratio,
    Representation: Copy + NumCast + NumOps + MulExact,
{
    type Error = core::convert::Infallible;

    /// Default implementation of a "try" conversion whenever two time scales can already be
    /// converted infallibly.
    fn try_convert(
        from: TimePoint<From, Representation, Period>,
    ) -> Result<TimePoint<To, Representation, Period>, Self::Error>
    where
        (): IsValidConversion<i64, <From as TimeScale>::NativePeriod, Period>
            + IsValidConversion<i64, <To as TimeScale>::NativePeriod, Period>,
    {
        Ok(<() as TimeScaleConversion<From, To>>::transform(from))
    }
}
