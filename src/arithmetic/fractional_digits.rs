//! Implementation of the ability to obtain an iterator over the fractional digits of some duration
//! representation. Used to represent subseconds when printing.

use core::ops::{Mul, Neg, Rem};

use num_traits::{ConstZero, Float, Zero};

use crate::Fraction;

/// Represents the ability to iterate the fractional digits of some value.
pub trait FractionalDigits {
    type Iterator: Iterator<Item = u8>;

    /// Returns an iterator into the fractional digits that make up a given number.
    fn fractional_digits(
        self,
        unit_ratio: Fraction,
        precision: Option<usize>,
        base: u8,
    ) -> Self::Iterator;
}

// Back-up limit that is used to prevent infinite loops while printing. The value of this constant
// should not be relied upon for any practical reasons other than preventing infinite loops.
const ABSOLUTE_MAX_DIGITS: usize = 64;

macro_rules! impl_fractional_digits_for_unsigned {
    ($repr:ty) => {
        impl FractionalDigits for $repr {
            type Iterator = FractionalDigitsIterator;

            fn fractional_digits(
                self,
                unit_ratio: Fraction,
                precision: Option<usize>,
                base: u8,
            ) -> Self::Iterator {
                FractionalDigitsIterator::from_unsigned(self, unit_ratio, precision, base)
            }
        }
    };
}

macro_rules! impl_fractional_digits_for_signed {
    ($repr:ty) => {
        impl FractionalDigits for $repr {
            type Iterator = FractionalDigitsIterator;

            fn fractional_digits(
                self,
                unit_ratio: Fraction,
                precision: Option<usize>,
                base: u8,
            ) -> Self::Iterator {
                FractionalDigitsIterator::from_signed(self, unit_ratio, precision, base)
            }
        }
    };
}

macro_rules! impl_fractional_digits_for_float {
    ($repr:ty) => {
        impl FractionalDigits for $repr {
            type Iterator = FractionalDigitsIterator;

            fn fractional_digits(
                self,
                unit_ratio: Fraction,
                precision: Option<usize>,
                base: u8,
            ) -> Self::Iterator {
                FractionalDigitsIterator::from_float(self, unit_ratio, precision, base)
            }
        }
    };
}

impl_fractional_digits_for_unsigned!(u8);
impl_fractional_digits_for_unsigned!(u16);
impl_fractional_digits_for_unsigned!(u32);
impl_fractional_digits_for_unsigned!(u64);
impl_fractional_digits_for_unsigned!(u128);
impl_fractional_digits_for_signed!(i8);
impl_fractional_digits_for_signed!(i16);
impl_fractional_digits_for_signed!(i32);
impl_fractional_digits_for_signed!(i64);
impl_fractional_digits_for_signed!(i128);
impl_fractional_digits_for_float!(f32);
impl_fractional_digits_for_float!(f64);

/// Wrapper struct that implements `FractionalDigits` for all integers.
pub struct FractionalDigitsIterator {
    remainder: u128,
    denominator: u128,
    base: u8,
    precision: Option<usize>,
    current_digits: usize,
}

impl FractionalDigitsIterator {
    pub fn from_signed<T>(count: T, fraction: Fraction, precision: Option<usize>, base: u8) -> Self
    where
        T: Copy
            + TryInto<u128>
            + Mul<T, Output = T>
            + Rem<T, Output = T>
            + ConstZero
            + PartialOrd
            + Neg<Output = T>,
    {
        let count = if count >= T::ZERO { count } else { -count }
            .try_into()
            .unwrap_or_else(|_| panic!());
        let numerator = (fraction.numerator() as u128) * count;
        let denominator = fraction.denominator().into();
        Self {
            remainder: numerator % denominator,
            denominator,
            base,
            precision,
            current_digits: 0,
        }
    }

    pub fn from_unsigned<T>(
        count: T,
        fraction: Fraction,
        precision: Option<usize>,
        base: u8,
    ) -> Self
    where
        T: Copy + Into<u128> + Mul<T, Output = T> + Rem<T, Output = T> + ConstZero + PartialOrd,
    {
        let numerator = (fraction.numerator() as u128) * count.into();
        let denominator = fraction.denominator().into();
        Self {
            remainder: numerator % denominator,
            denominator,
            base,
            precision,
            current_digits: 0,
        }
    }

    pub fn from_float<T>(count: T, fraction: Fraction, precision: Option<usize>, base: u8) -> Self
    where
        T: Mul<Fraction, Output = T> + Float,
    {
        let (mantissa, exponent, _) = count.integer_decode();
        let mut numerator = (mantissa as u128) * (fraction.numerator() as u128);
        let mut denominator = fraction.denominator() as u128;
        if exponent > 0 {
            numerator *= 2u128.pow(exponent as u32)
        } else if exponent < 0 {
            denominator *= 2u128.pow(-exponent as u32)
        };

        Self {
            remainder: numerator % denominator,
            denominator,
            base,
            precision,
            current_digits: 0,
        }
    }
}

impl Iterator for FractionalDigitsIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let keep_going = if let Some(precision) = self.precision {
            self.current_digits < precision
        } else {
            !self.remainder.is_zero()
        };

        if keep_going && self.current_digits < ABSOLUTE_MAX_DIGITS {
            self.current_digits += 1;
            self.remainder *= self.base as u128;
            let digit: u8 = (self.remainder / self.denominator)
                .try_into()
                .unwrap_or_else(|_| panic!());
            self.remainder %= self.denominator;
            Some(digit)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn integer_fractions() {
    let fraction: Vec<_> = 7854i32
        .fractional_digits(Fraction::new(1, 1_000), Some(8), 10)
        .collect();
    assert_eq!(fraction, vec![8, 5, 4, 0, 0, 0, 0, 0]);

    let fraction: Vec<_> = 1_234_567_890_123i64
        .fractional_digits(Fraction::new(1, 1_000_000_000_000), Some(9), 10)
        .collect();
    assert_eq!(fraction, vec![2, 3, 4, 5, 6, 7, 8, 9, 0]);
}

#[cfg(feature = "std")]
#[test]
fn float_fractions() {
    let fraction: Vec<_> = 7_854f32
        .fractional_digits(Fraction::new(1, 1_000), Some(8), 10)
        .collect();
    assert_eq!(fraction, vec![8, 5, 4, 0, 0, 0, 0, 0]);

    let fraction: Vec<_> = 1_234_567_890_123f64
        .fractional_digits(Fraction::new(1, 1_000_000_000_000), Some(9), 10)
        .collect();
    assert_eq!(fraction, vec![2, 3, 4, 5, 6, 7, 8, 9, 0]);
}
