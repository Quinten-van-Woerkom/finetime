//! Implementation of the ability to obtain an iterator over the fractional digits of some duration
//! representation. Used to represent subseconds when printing.

use core::ops::{Div, Mul, MulAssign, Rem, RemAssign};

use num_traits::{ConstOne, ConstZero, One};

use crate::Fraction;

/// Represents the ability to iterate the fractional digits of some value.
pub trait FractionalDigits {
    type Iterator: Iterator<Item = u8>;

    /// Returns an iterator into the fractional digits that make up a given number.
    fn fractional_digits(self, unit_ratio: Fraction, precision: Option<usize>) -> Self::Iterator;
}

impl<T> FractionalDigits for T
where
    T: Copy
        + TryFrom<u64>
        + TryInto<u8>
        + ConstZero
        + ConstOne
        + One
        + MulAssign
        + Mul<T, Output = T>
        + RemAssign
        + Rem<T, Output = T>
        + Div<T, Output = T>
        + PartialEq,
{
    type Iterator = IntegerFractionalDigits<T>;

    fn fractional_digits(self, unit_ratio: Fraction, precision: Option<usize>) -> Self::Iterator {
        IntegerFractionalDigits::new(self, unit_ratio, precision)
    }
}

// impl_fractional_digits!(u8);
// impl_fractional_digits!(u16);
// impl_fractional_digits!(u32);
// impl_fractional_digits!(u64);
// impl_fractional_digits!(u128);
// impl_fractional_digits!(i8);
// impl_fractional_digits!(i16);
// impl_fractional_digits!(i32);
// impl_fractional_digits!(i64);
// impl_fractional_digits!(i128);

/// Wrapper struct that implements `FractionalDigits` for all integers.
pub struct IntegerFractionalDigits<T> {
    remainder: T,
    denominator: T,
    precision: Option<usize>,
    current_digits: usize,
}

impl<T> IntegerFractionalDigits<T>
where
    T: Copy + TryFrom<u64> + Mul<T, Output = T> + Rem<T, Output = T>,
{
    pub fn new(count: T, fraction: Fraction, precision: Option<usize>) -> Self {
        let fraction_numerator: T = fraction.numerator().try_into().unwrap_or_else(|_| panic!());
        let numerator = fraction_numerator * count;
        let denominator = fraction
            .denominator()
            .try_into()
            .unwrap_or_else(|_| panic!());
        Self {
            remainder: numerator % denominator,
            denominator,
            precision,
            current_digits: 0,
        }
    }
}

impl<T> Iterator for IntegerFractionalDigits<T>
where
    T: Copy
        + TryFrom<u64>
        + TryInto<u8>
        + ConstZero
        + ConstOne
        + One
        + MulAssign
        + RemAssign
        + Div<T, Output = T>
        + PartialEq,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let ten = (T::ONE + T::ONE + T::ONE + T::ONE + T::ONE) * (T::ONE + T::ONE);
        let keep_going = if let Some(precision) = self.precision {
            self.current_digits < precision
        } else {
            !self.remainder.is_zero()
        };
        // Back-up limit that is used to prevent infinite loops.
        const ABSOLUTE_MAX_DIGITS: usize = 64;
        if keep_going && self.current_digits < ABSOLUTE_MAX_DIGITS {
            self.current_digits += 1;
            self.remainder *= ten;
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

#[test]
fn integer_fractions() {
    let fraction: Vec<_> = 7854
        .fractional_digits(Fraction::new(1, 1_000), Some(8))
        .collect();
    assert_eq!(fraction, vec![8, 5, 4, 0, 0, 0, 0, 0]);

    let fraction: Vec<_> = 1_234_567_890_123i64
        .fractional_digits(Fraction::new(1, 1_000_000_000_000), Some(9))
        .collect();
    assert_eq!(fraction, vec![2, 3, 4, 5, 6, 7, 8, 9, 0]);
}
