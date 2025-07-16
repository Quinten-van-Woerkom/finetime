//! Time scales are used to relate time points with each other. They are, in effect, a manner to
//! link instances in time to a number of elapsed seconds since some epoch. The manner in which
//! this relation is established differs per time scale.

use core::ops::{Add, Div, Mul, Sub};

use num::{NumCast, traits::NumOps};

use crate::{
    duration::{
        Duration, Hours, Minutes, Seconds,
        units::{IsValidConversion, LiteralRatio, Milli, Ratio},
    },
    time_point::{DateTimeError, TimePoint},
    time_scale::{local::LocalDays, tai::Tai},
};

pub mod local;
pub mod tai;
pub mod unix;
pub mod utc;

/// A time scale is a specification for measuring time. In this implementation, we specify this by
/// relating times to an elapsed duration since some reference epoch.
///
/// Additionally, all `TimeScale`s shall be able to convert `TimePoint`s from their scale to TAI
/// and the other way around. This is used to fundamentally connect all clocks.
pub trait TimeScale: Sized {
    /// Returns the reference epoch of a time scale, expressed in number of seconds since the TAI
    /// epoch.
    fn reference_epoch() -> TimePoint<Tai, i64, Milli>;

    /// Returns the epoch of a time scale, expressed as a `LocalTime` in its own time scale. The
    /// result may be expressed in any type `T`, as long as this type can be constructed from some
    /// primitive. This function is allowed to panic if the epoch, expressed as `LocalDays`, cannot
    /// be represented by a value of type `T`.
    fn epoch<T>() -> LocalDays<T>
    where
        T: NumCast;

    /// Returns whether this time scales incorporates leap seconds, i.e., whether the underlying
    /// "seconds since epoch" count also increases one second when a leap second is inserted.
    fn counts_leap_seconds() -> bool;

    /// Creates a `TimePoint` from some previously created `LocalDays` instance by adding a given
    /// time-of-day to it.
    fn from_local_datetime<Representation>(
        date: LocalDays<Representation>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, Representation>, DateTimeError<Representation>>
    where
        Representation: NumCast
            + Sub<Representation, Output = Representation>
            + Add<Representation, Output = Representation>
            + Mul<Representation, Output = Representation>
            + Div<Representation, Output = Representation>
            + Clone,
        (): IsValidConversion<Representation, LiteralRatio<86400>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<3600>, LiteralRatio<1>>
            + IsValidConversion<Representation, LiteralRatio<60>, LiteralRatio<1>>,
    {
        // First, we verify that the timestamp is valid.
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(DateTimeError::InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }

        // Afterwards, we convert the date to its MJD equivalent. We do the same for the TAI epoch,
        // but then at compile time already. Note that both dates are MJD, expressed in TAI.
        let date_mjd = date;
        let tai_epoch = Self::epoch::<Representation>();
        let days = date_mjd - tai_epoch;
        // The following casts will always succeed for primitive types, because 0..=60
        let hours = Hours::new(hour).cast().unwrap();
        let minutes = Minutes::new(minute).cast().unwrap();
        let seconds = Seconds::new(second).cast().unwrap();
        Ok(TimePoint::from_time_since_epoch(
            days.convert() + hours.convert() + minutes.convert() + seconds,
        ))
    }

    /// Creates a time point in this time scale based on a time point in TAI. Note that some
    /// rounding is permitted to occur here: not all time scales can be related exactly to TAI.
    fn from_tai<Representation, Period>(
        time_point: TimePoint<Tai, Representation, Period>,
    ) -> TimePoint<Self, Representation, Period>
    where
        (): TimeScaleConversion<Tai, Self>,
        Period: Ratio,
        Representation: Copy + NumCast + NumOps,
    {
        <() as TimeScaleConversion<Tai, Self>>::convert(time_point)
    }

    /// Creates a TAI time point based on a time point in this time scale. Rounding is permitted,
    /// as not all time scales can be exactly related to TAI.
    fn to_tai<Representation, Period>(
        time_point: TimePoint<Self, Representation, Period>,
    ) -> TimePoint<Tai, Representation, Period>
    where
        (): TimeScaleConversion<Self, Tai>,
        Period: Ratio,
        Representation: Copy + NumCast + NumOps,
    {
        <() as TimeScaleConversion<Self, Tai>>::convert(time_point)
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
    fn convert<Representation, Period>(
        from: TimePoint<From, Representation, Period>,
    ) -> TimePoint<To, Representation, Period>
    where
        Period: Ratio,
        Representation: Copy + NumCast + NumOps,
    {
        let time_since_from_epoch = from.elapsed_time_since_epoch();
        let from_epoch = From::reference_epoch();
        let to_epoch = To::reference_epoch();
        // Note that this operation first rounds and then casts the epoch differences into the
        // proper units and representation. The representation cast may fail, if the difference in
        // epochs is not representable by the chosen representation (e.g., a `u8` cannot store the
        // number of seconds between the `To` and `From` epoch). In such cases, this conversion will
        // panic.
        let epoch_difference: Duration<Representation, Period> =
            (to_epoch - from_epoch).round().cast().unwrap();
        TimePoint::<To, Representation, Period>::from_time_since_epoch(
            time_since_from_epoch - epoch_difference,
        )
    }
}

impl<T: TimeScale> TimeScaleConversion<T, T> for () {
    /// Conversion from a clock to itself is always possible and a no-op.
    fn convert<Representation, Period>(
        from: TimePoint<T, Representation, Period>,
    ) -> TimePoint<T, Representation, Period> {
        from
    }
}
