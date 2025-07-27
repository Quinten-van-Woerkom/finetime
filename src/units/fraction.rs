//! This file contains all logic related to `Fraction`s and operations on them.

use core::ops::{Div, Mul};

use num::{Bounded, Integer, NumCast};

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
    #[cfg(kani)]
    pub(crate) const fn numerator(&self) -> i128 {
        self.numerator
    }

    /// Returns the value of this fraction's denominator.
    #[cfg(kani)]
    pub(crate) const fn denominator(&self) -> i128 {
        self.denominator
    }
}

/// Fractions may be multiplied with any type that supports conversion from `i128` using
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
        T: Integer + MulExact,
    {
        // For integers, multiplication with rounding to nearest is exactly what is done in the
        // underlying `MulExact` implementation.
        rhs.mul_exact(self)
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
        intermediate.div_floor(&denominator)
    }
}

/// Trait that represents exact multiplication by a fraction. If the result must be an integer,
/// rounds to the nearest representable value of the input type. Useful in cases where an integer
/// is multiplied by a non-integer value but must be exact over its entire input range.
pub trait MulExact {
    fn mul_exact(self, fraction: Fraction) -> Self;
}

/// Generic implementation of `mul_round` that can be applied to any type that may be losslessly
/// converted into a `i128`. Note that, due to trait coherence, this is not directly an
/// implementation of `MulRound`: that must be added manually for each relevant type.
///
/// This implementation may not be used for `i128` and `u128` themselves, since then overflow might
/// occur: instead, those rely on an implementation that makes use of `i256`.
fn mul_exact<T>(value: T, fraction: Fraction) -> T
where
    T: NumCast + Integer + Copy,
{
    let numerator = fraction.numerator;
    let denominator = fraction.denominator;
    let value: i128 = value.to_i128().unwrap();
    let intermediate = value * numerator;
    let (result, remainder) = intermediate.div_rem(&denominator);
    let twice = remainder + remainder;

    let result = if denominator > 0i128 && twice >= denominator {
        result + 1i128
    } else if denominator < 0i128 && twice <= denominator {
        result - 1i128
    } else {
        result
    };
    T::from(result).unwrap()
}

/// Generic implementation of `mul_round` for big integers like `u128` and `i128` that could
/// overflow beyond `i128` if multiplied by a fraction.
fn mul_round_bigint<T>(value: T, fraction: Fraction) -> T
where
    T: NumCast + Integer + Copy,
    i256::i256: From<T>,
{
    use i256::i256;
    let rhs: i256 = value.into();
    let numerator: i256 = fraction.numerator.into();
    let denominator: i256 = fraction.denominator.into();
    let intermediate: i256 = rhs * numerator;
    let (result, remainder) = intermediate.div_rem(denominator);
    let twice = remainder + remainder;

    use num::traits::{One, Zero};
    let one = i256::one();
    let zero = i256::zero();
    let result = if denominator > zero && twice >= denominator {
        result + one
    } else if denominator < zero && twice <= denominator {
        result - one
    } else {
        result
    };
    T::from(result).unwrap()
}

impl MulExact for f32 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        fraction * self
    }
}

impl MulExact for f64 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        fraction * self
    }
}

impl MulExact for i8 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for i16 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for i32 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for i64 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for i128 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_round_bigint(self, fraction)
    }
}

impl MulExact for u8 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for u16 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for u32 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for u64 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_exact(self, fraction)
    }
}

impl MulExact for u128 {
    fn mul_exact(self, fraction: Fraction) -> Self {
        mul_round_bigint(self, fraction)
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
