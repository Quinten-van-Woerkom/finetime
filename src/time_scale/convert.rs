//! Logic related to conversions between time scales.

use crate::TimePoint;

/// Trait representing the ability to convert from one scale into another. Note that this
/// conversion must always succeed: barring arithmetic overflows (on which panics are advised),
/// it must always be possible to relate the time of two scales.
///
/// Note that this trait must be implemented on a given `TimePoint`. The converse trait,
/// `IntoScale`, may be derived automatically. Just like with `From` and `Into`, it is advised to
/// simply implement `FromScale` and let `IntoScale` be derived.
pub trait FromTimeScale<Scale, Representation, Period> {
    /// Constructs a time point from an instant expressed in another scale.
    fn from_time_scale(time_point: TimePoint<Scale, Representation, Period>) -> Self;
}

/// Trait representing the ability to convert from one scale into another. Note that this
/// conversion must always succeed: barring arithmetic overflows (on which panics are advised),
/// it must always be possible to relate the time of two scales.
///
/// This trait shall generally be derived based on an existing `FromScale` implementation.
pub trait IntoTimeScale<Scale, Representation, Period> {
    /// Constructs a time point from an instant expressed in another scale.
    fn into_time_scale(self) -> TimePoint<Scale, Representation, Period>;
}

impl<S1, R1, P1, S2, R2, P2> IntoTimeScale<S1, R1, P1> for TimePoint<S2, R2, P2>
where
    TimePoint<S1, R1, P1>: FromTimeScale<S2, R2, P2>,
{
    fn into_time_scale(self) -> TimePoint<S1, R1, P1> {
        TimePoint::from_time_scale(self)
    }
}
