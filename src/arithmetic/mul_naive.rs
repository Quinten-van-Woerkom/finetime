//! Implementation of round-to-nearest, even-on-tie multiplication. For floating point numbers,
//! this is just regular multiplication.

use core::ops::{Div, Mul};

use num_traits::cast::FromPrimitive;

use crate::arithmetic::Fraction;

/// Trait that represents naive multiplication by a fraction. This multiplication simply accepts
/// truncating multiplications: no checks are done to verify whether that is accurate.
pub trait MulNaive {
    fn mul_naive(self, fraction: Fraction) -> Self;
}

impl<T> MulNaive for T
where
    T: Mul<T, Output = T> + Div<T, Output = T> + FromPrimitive,
{
    fn mul_naive(self, fraction: Fraction) -> Self {
        let numerator: T = T::from_i128(fraction.numerator()).unwrap();
        let denominator: T = T::from_i128(fraction.denominator()).unwrap();
        (self * numerator) / denominator
    }
}
