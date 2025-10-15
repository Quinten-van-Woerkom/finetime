//! Supporting code for common arithmetic operations: casting, converting, fractions, etc.

mod fraction;
pub use fraction::{Fraction, MulCeil, MulFloor, MulRound, TryMul};
mod fractional_digits;
pub use fractional_digits::FractionalDigits;
mod try_exact;
pub use try_exact::{TryFromExact, TryIntoExact};
