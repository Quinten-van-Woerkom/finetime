//! This file implements the concept of a "terrestrial time", referring to any time scale which
//! represents the Platonic ideal of a time scale representing the elapsed time on the Earth geoid.

use core::{
    convert::From,
    ops::{Add, Sub},
};

use crate::{
    ConvertUnit, Duration, FromTimeScale, TimePoint, TryFromExact, time_scale::AbsoluteTimeScale,
    units::SecondsPerDay,
};

/// In general, "terrestrial time" refers not just to the specific realization TT, but to an
/// idealized clock on the Earth geoid. It turns out that a lot of time scales are simply a variant
/// on terrestrial time (or, equivalently, TAI). All these time scales may easily be converted into
/// one another through a simple epoch offset: their internal clock rates are identical.
pub trait TerrestrialTime: AbsoluteTimeScale {
    /// The underlying representation used to represent the offset with respect to TAI. For
    /// compatibility with as wide a range of `TimePoint` types, it's best to make this as small a
    /// type as possible (e.g., u8, u32, etc.).
    type Representation;

    /// The "native" period used to represent the offset with respect to TAI. For wide support, it
    /// is best to choose the largest possible unit.
    type Period;

    const TAI_OFFSET: Duration<Self::Representation, Self::Period>;
}

impl<ScaleFrom, ScaleInto, Representation, Period> FromTimeScale<ScaleFrom, Representation, Period>
    for TimePoint<ScaleInto, Representation, Period>
where
    ScaleFrom: TerrestrialTime,
    ScaleInto: TerrestrialTime,
    Representation: Copy
        + Add<Representation, Output = Representation>
        + Sub<Representation, Output = Representation>
        + From<ScaleFrom::Representation>
        + From<ScaleInto::Representation>
        + TryFromExact<i32>
        + ConvertUnit<ScaleFrom::Period, Period>
        + ConvertUnit<ScaleInto::Period, Period>
        + ConvertUnit<SecondsPerDay, Period>
        + PartialOrd,
{
    fn from_time_scale(time_point: TimePoint<ScaleFrom, Representation, Period>) -> Self {
        let epoch_offset = ScaleFrom::EPOCH.elapsed_calendar_days_since(ScaleInto::EPOCH);
        let epoch_offset = epoch_offset
            .try_cast()
            .unwrap_or_else(|_| panic!())
            .into_unit();
        let from_offset: Duration<Representation, Period> =
            ScaleFrom::TAI_OFFSET.cast().into_unit();
        let into_offset: Duration<Representation, Period> =
            ScaleInto::TAI_OFFSET.cast().into_unit();
        // Depending on the sign, we flip the subtraction order. This is useful to ensure that we
        // do not overflow past zero for unsigned integers, and to keep the integer range needed as
        // small as possible in general.
        let time_since_epoch = if from_offset >= into_offset {
            let scale_offset = from_offset - into_offset;
            time_point.time_since_epoch() - scale_offset + epoch_offset
        } else {
            let scale_offset = into_offset - from_offset;
            time_point.time_since_epoch() + scale_offset + epoch_offset
        };
        Self::from_time_since_epoch(time_since_epoch)
    }
}
