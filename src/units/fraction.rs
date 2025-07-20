//! This file contains all logic related to `Fraction`s and operations on them.

use core::ops::{Div, Mul};

use num::{Bounded, Integer, NumCast};

/// Description of an integer ratio. Written to support efficient compile-time arithmetic. To
/// support conversions between large magnitudes, this is implemented in u64. The numerator may be
/// 0, but the denominator may never be.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fraction {
    numerator: u64,
    denominator: u64,
}

impl Fraction {
    /// Creates a new `Ratio` with the given values for `numerator` and `denominator`. Is
    /// normalized to the smallest possible representation.
    pub(crate) const fn new(numerator: u64, denominator: u64) -> Self {
        if denominator == 0 {
            panic!("Created invalid ratio with denominator 0.");
        } else {
            Self {
                numerator,
                denominator,
            }
            .normalized()
        }
    }

    /// `Ratio`s will always be stored in normalized fashion, to ensure that equality is simple
    /// bitwise equality, and to prevent integer overflow from occuring.
    pub(crate) const fn normalized(&self) -> Self {
        let gcd = binary_gcd(self.numerator, self.denominator);
        Self {
            numerator: self.numerator / gcd,
            denominator: self.denominator / gcd,
        }
    }

    /// Const implementation of division. Used to determine the conversion factor needed to
    /// transform between two units. Applies the GCD algorithm three times to keep the numbers as
    /// small as possible - as needed to prevent overflow even for large conversion ratios.
    ///
    /// While the repeated application of GCD might seem to be expensive for a simple division,
    /// this function is only expected to be used at runtime. Consequently, the expense should not
    /// be a problem.
    pub(crate) const fn divide_by(&self, other: &Self) -> Self {
        let gcd1 = binary_gcd(self.numerator, other.numerator);
        let gcd2 = binary_gcd(self.denominator, other.denominator);
        let numerator = (self.numerator / gcd1) * (other.denominator / gcd2);
        let denominator = (self.denominator / gcd2) * (other.numerator / gcd1);
        let gcd3 = binary_gcd(numerator, denominator);
        Self {
            numerator: numerator / gcd3,
            denominator: denominator / gcd3,
        }
    }

    /// Returns the reciprocal value of this fraction.
    #[cfg(kani)]
    pub(crate) const fn recip(&self) -> Self {
        Self::new(self.denominator, self.numerator)
    }

    /// Returns the value of this fraction's numerator.
    #[cfg(kani)]
    pub(crate) const fn numerator(&self) -> u64 {
        self.numerator
    }

    /// Returns the value of this fraction's denominator.
    #[cfg(kani)]
    pub(crate) const fn denominator(&self) -> u64 {
        self.denominator
    }
}

/// Fractions may be multiplied with any type that supports conversion to and from `u64` using
/// `num_trait::NumCast`. This kind of casting permits rounding errors, but returns an `Error` if
/// other problems, like range errors, are encountered.
impl<T> Mul<T> for Fraction
where
    T: NumCast + Mul<T, Output = T> + Div<T, Output = T>,
{
    type Output = T;

    fn mul(self, rhs: T) -> Self::Output {
        let numerator: T = T::from(self.numerator).unwrap();
        let denominator: T = T::from(self.denominator).unwrap();
        (rhs * numerator) / denominator
    }
}

impl Fraction {
    /// Multiplies with the given `Integer`, but checks if the resulting value is exact or results
    /// in a truncating division. Note that this function may still panic if the multiplication by
    /// the numerator overflows.
    #[cfg_attr(kani, kani::requires(rhs <= T::max_value() / T::from(self.numerator).unwrap()))]
    #[cfg_attr(kani, kani::requires(rhs >= T::min_value() / T::from(self.numerator).unwrap()))]
    pub(crate) fn try_mul<T>(self, rhs: T) -> Option<T>
    where
        T: NumCast + Integer + Copy + Bounded,
    {
        let numerator: T = T::from(self.numerator).unwrap();
        let denominator: T = T::from(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        let (result, remainder) = intermediate.div_rem(&denominator);
        if remainder.is_zero() {
            Some(result)
        } else {
            None
        }
    }

    /// Multiplies the right-hand side `Integer` by this fraction. Rounds towards the nearest
    /// integer if the result is not an integer value.
    pub(crate) fn mul_round<T>(self, rhs: T) -> T
    where
        T: NumCast + Integer + Copy,
    {
        let numerator: T = T::from(self.numerator).unwrap();
        let denominator: T = T::from(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        let (result, remainder) = intermediate.div_rem(&denominator);
        let twice = remainder + remainder;

        if denominator > T::zero() && twice >= denominator {
            result + T::one()
        } else if denominator < T::zero() && twice <= denominator {
            result - T::one()
        } else {
            result
        }
    }

    /// Multiplies the right-hand side `Integer` by this fraction. Rounds towards positive infinity
    /// if the result is not an integer value.
    pub(crate) fn mul_ceil<T>(self, rhs: T) -> T
    where
        T: NumCast + Integer + Copy,
    {
        let numerator: T = T::from(self.numerator).unwrap();
        let denominator: T = T::from(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        intermediate.div_ceil(&denominator)
    }

    /// Multiplies the right-hand side `Integer` by this fraction. Rounds towards negative infinity
    /// if the result is not an integer value.
    pub(crate) fn mul_floor<T>(self, rhs: T) -> T
    where
        T: NumCast + Integer + Copy,
    {
        let numerator: T = T::from(self.numerator).unwrap();
        let denominator: T = T::from(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        intermediate.div_ceil(&denominator)
    }
}

/// Returns the greatest common denominator of `a` and `b`, computed using Stein's algorithm.
const fn binary_gcd(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        panic!("GCD is undefined when one of the arguments is zero");
    }

    let (mut a, mut b) = (a, b);
    let (mut u, mut v) = (a, b);

    // Count common powers of two
    let i = u.trailing_zeros();
    let j = v.trailing_zeros();
    u >>= i;
    v >>= j;
    let k = if i < j { i } else { j };

    loop {
        if u > v {
            let temp = u;
            u = v;
            v = temp;
            let temp = a;
            a = b;
            b = temp;
        } else if u == v {
            return u << k;
        }

        v -= u;

        v >>= v.trailing_zeros();
    }
}
