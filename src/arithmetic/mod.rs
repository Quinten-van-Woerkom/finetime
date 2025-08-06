//! Implementation of auxiliary arithmetic functionality. Useful because implementing exact
//! `Duration` and `TimePoint` arithmetic requires quite some additional non-standard functionality
//! of their underlying representation types that are not well-captured using default traits and
//! functionality.

mod fraction;
pub use fraction::*;
mod units;
pub use units::*;
mod div_ceil;
pub use div_ceil::*;
mod div_floor;
pub use div_floor::*;
mod mul_round;
pub use mul_round::*;
mod mul_naive;
pub use mul_naive::*;
mod try_from_exact;
pub use try_from_exact::*;

use num_traits::{FromPrimitive, NumOps, One, Zero};

/// Auxiliary trait that describes all functionality that is required of a time implementation.
/// Primarily used to reduce the size of the `where` clauses in generic implementations, but also
/// helpful as implementation guide when implementing a custom time representation.
pub trait TimeRepresentation:
    Clone
    + NumOps
    + MulExact
    + MulRound
    + MulNaive
    + DivCeil
    + DivFloor
    + Zero
    + One
    + PartialOrd
    + FromPrimitive
    + TryFromExact<i64>
    + TryIntoExact<i64>
    + TryFromExact<u8>
{
}

impl<T> TimeRepresentation for T where
    T: Clone
        + NumOps
        + MulExact
        + MulNaive
        + MulRound
        + DivCeil
        + DivFloor
        + Zero
        + One
        + PartialOrd
        + FromPrimitive
        + TryFromExact<i64>
        + TryIntoExact<i64>
        + TryFromExact<u8>
{
}
