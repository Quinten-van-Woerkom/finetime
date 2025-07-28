//! This file contains all logic related to `Fraction`s and operations on them.

use num::{FromPrimitive, Integer, traits::NumOps};

use crate::arithmetic::{DivCeil, DivFloor, TryFromExact};

/// Description of an integer ratio. Written to support efficient compile-time arithmetic. To
/// support conversions between large magnitudes, this is implemented in i128. The numerator may be
/// 0, but the denominator may never be.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fraction {
    numerator: i128,
    denominator: i128,
}

impl Fraction {
    /// Creates a new `Ratio` with the given values for `numerator` and `denominator`. Is
    /// normalized to the smallest possible representation.
    pub(crate) const fn new(numerator: i128, denominator: i128) -> Self {
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
    pub(crate) const fn numerator(&self) -> i128 {
        self.numerator
    }

    /// Returns the value of this fraction's denominator.
    pub(crate) const fn denominator(&self) -> i128 {
        self.denominator
    }

    /// Multiplies with the given `Integer`, but checks if the resulting value is exact or results
    /// in a truncating division. Note that this function may still panic if the multiplication by
    /// the numerator overflows.
    pub(crate) fn try_mul<T>(self, rhs: T) -> Option<T>
    where
        T: Integer + TryFromExact<i128>,
    {
        let numerator: T = T::try_from_exact(self.numerator).ok().unwrap();
        let denominator: T = T::try_from_exact(self.denominator).ok().unwrap();
        let intermediate = rhs * numerator;
        let (result, remainder) = intermediate.div_rem(&denominator);
        if remainder.is_zero() {
            Some(result)
        } else {
            None
        }
    }

    /// Multiplies the right-hand side `Integer` by this fraction. Rounds towards positive infinity
    /// if the result is not an integer value.
    pub(crate) fn mul_ceil<T>(self, rhs: T) -> T
    where
        T: FromPrimitive + NumOps + DivCeil,
    {
        let numerator: T = T::from_i128(self.numerator).unwrap();
        let denominator: T = T::from_i128(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        intermediate.div_ceil(&denominator)
    }

    /// Multiplies the right-hand side number by this fraction. Rounds towards negative infinity
    /// if the result is not an integer value.
    pub(crate) fn mul_floor<T>(self, rhs: T) -> T
    where
        T: FromPrimitive + NumOps + DivFloor,
    {
        let numerator: T = T::from_i128(self.numerator).unwrap();
        let denominator: T = T::from_i128(self.denominator).unwrap();
        let intermediate = rhs * numerator;
        intermediate.div_floor(&denominator)
    }
}

/// Returns the greatest common denominator of `a` and `b`, computed using Stein's algorithm.
const fn binary_gcd(a: i128, b: i128) -> i128 {
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
