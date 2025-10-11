//! Representation of the concept of a "terrestrial time", which means that a given time scale
//! expresses the rate of time found on the Earth geoid.

use crate::Duration;

/// A significant amount of time scales represent the time of an idealized clock on the Earth
/// geoid. Because all such scales represent the same underlying clock rate, conversion between
/// them is reduced to a simple epoch shift.
pub trait TerrestrialTime {
    /// The underlying type used for representing the offset between this clock and TAI.
    type Representation;

    /// The period used to express the offset between this clock and TAI.
    type Period;

    /// The (constant) offset between this clock and TAI. Positive durations imply that the clock
    /// is ahead of TAI, while negative durations mean that TAI is ahead.
    const TAI_OFFSET: Duration<Self::Representation, Self::Period>;
}
