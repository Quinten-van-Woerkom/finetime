//! This file implements the concept of a "terrestrial time", referring to any time scale which
//! represents the Platonic ideal of a time scale representing the elapsed time on the Earth geoid.

use core::ops::Sub;

use crate::{ConvertUnit, Duration, FromScale, TimePoint};

/// In general, "terrestrial time" refers not just to the specific realization TT, but to an
/// idealized clock on the Earth geoid. It turns out that a lot of time scales are simply a variant
/// on terrestrial time (or, equivalently, TAI). All these time scales may easily be converted into
/// one another through a simple epoch offset: their internal clock rates are identical.
pub trait TerrestrialTime {
    type Representation;

    type Period;

    const TAI_OFFSET: Duration<Self::Representation, Self::Period>;
}

impl<From, Into, Representation, Period> FromScale<From, Representation, Period>
    for TimePoint<Into, Representation, Period>
where
    From: TerrestrialTime,
    Into: TerrestrialTime,
    Representation: Copy
        + Sub<Representation, Output = Representation>
        + core::convert::From<From::Representation>
        + core::convert::From<Into::Representation>
        + ConvertUnit<From::Period, Period>
        + ConvertUnit<Into::Period, Period>,
{
    fn from_scale(time_point: TimePoint<From, Representation, Period>) -> Self {
        let from_offset: Duration<Representation, Period> = From::TAI_OFFSET.cast().into_unit();
        let into_offset: Duration<Representation, Period> = Into::TAI_OFFSET.cast().into_unit();
        let offset = from_offset - into_offset;
        let time_since_epoch = time_point.time_since_epoch() - offset;
        Self::from_time_since_epoch(time_since_epoch)
    }
}
