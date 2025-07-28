//! Implementation of round-to-nearest, even-on-tie multiplication. For floating point numbers,
//! this is just regular multiplication.

use i256::i256;
use num::{Integer, ToPrimitive};

use crate::arithmetic::Fraction;

/// Trait that represents rounding multiplication by a fraction. If the result must be an integer,
/// rounds to the nearest representable value of the input type. Useful in cases where an integer
/// is multiplied by a non-integer value.
///
/// May still panic if the end result is not representable by the output type, e.g., if overflow
/// occurs.
pub trait MulRound {
    fn mul_round(self, fraction: Fraction) -> Self;
}

/// Generic implementation of `mul_round` that can be applied to any type that may be losslessly
/// converted into a `i128`. Note that, due to trait coherence, this is not directly an
/// implementation of `MulRound`: that must be added manually for each relevant type.
///
/// This implementation may not be used for `i128` and `u128` themselves, since then overflow might
/// occur: instead, those rely on an implementation that makes use of `i256`.
fn mul_round(value: i128, fraction: Fraction) -> i128 {
    let numerator = fraction.numerator();
    let denominator = fraction.denominator();
    let intermediate = value * numerator;
    let (result, remainder) = intermediate.div_rem(&denominator);
    let twice = remainder + remainder;

    if denominator > 0i128 && twice >= denominator {
        result + 1i128
    } else if denominator < 0i128 && twice <= denominator {
        result - 1i128
    } else {
        result
    }
}

/// Generic implementation of `mul_round` for big integers like `u128` and `i128` that could
/// overflow beyond `i128` if multiplied by a fraction.
fn mul_round_bigint(value: i256, fraction: Fraction) -> i256 {
    let numerator: i256 = fraction.numerator().into();
    let denominator: i256 = fraction.denominator().into();
    let intermediate: i256 = value * numerator;
    let (result, remainder) = intermediate.div_rem(denominator);
    let twice = remainder + remainder;

    use num::traits::{One, Zero};
    let one = i256::one();
    let zero = i256::zero();

    if denominator > zero && twice >= denominator {
        result + one
    } else if denominator < zero && twice <= denominator {
        result - one
    } else {
        result
    }
}

impl MulRound for f32 {
    fn mul_round(self, fraction: Fraction) -> Self {
        use crate::arithmetic::MulNaive;
        self.mul_naive(fraction)
    }
}

impl MulRound for f64 {
    fn mul_round(self, fraction: Fraction) -> Self {
        use crate::arithmetic::MulNaive;
        self.mul_naive(fraction)
    }
}

impl MulRound for i8 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for i16 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for i32 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for i64 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for i128 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round_bigint(self.into(), fraction).to_i128().unwrap()
    }
}

impl MulRound for u8 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for u16 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for u32 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for u64 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round(self.into(), fraction).try_into().unwrap()
    }
}

impl MulRound for u128 {
    fn mul_round(self, fraction: Fraction) -> Self {
        mul_round_bigint(self.into(), fraction).to_u128().unwrap()
    }
}
