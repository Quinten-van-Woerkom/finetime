//! This file contains all logic related to `Fraction`s and operations on them.

use core::ops::Mul;

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
    pub const fn new(numerator: u64, denominator: u64) -> Self {
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

    /// Returns the value of this fraction's numerator.
    pub const fn numerator(&self) -> u64 {
        self.numerator
    }

    /// Returns the value of this fraction's denominator.
    pub const fn denominator(&self) -> u64 {
        self.denominator
    }

    /// `Ratio`s will always be stored in normalized fashion, to ensure that equality is simple
    /// bitwise equality, and to prevent integer overflow from occuring.
    pub const fn normalized(&self) -> Self {
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
    /// this function is only expected to be used at compile time. Consequently, the expense should
    /// not be a problem.
    pub const fn divide_by(&self, other: &Self) -> Self {
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

impl Mul<f64> for Fraction {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        let numerator = self.numerator() as f64;
        let denominator = self.denominator() as f64;
        rhs * (numerator / denominator)
    }
}

impl Mul<f32> for Fraction {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        let numerator = self.numerator() as f32;
        let denominator = self.denominator() as f32;
        rhs * (numerator / denominator)
    }
}

impl Mul<Fraction> for f32 {
    type Output = f32;

    fn mul(self, rhs: Fraction) -> Self::Output {
        rhs * self
    }
}

impl Mul<Fraction> for f64 {
    type Output = f64;

    fn mul(self, rhs: Fraction) -> Self::Output {
        rhs * self
    }
}

/// Trait representing a fallible multiplication that fails if the result cannot be represented by
/// the output type without rounding error. "Small" errors like floating point error are
/// permissible.
pub trait TryMul<T> {
    type Output;

    /// Fallible multiplication, applicable in scenarios like fractions where multiplication by a
    /// fraction might not result in a value that can be represented in the original type
    /// (e.g., multiplication of an integer by a fraction can be non-integer).
    fn try_mul(self, rhs: T) -> Option<Self::Output>;
}

macro_rules! try_mul_integer {
    ( $repr:ty ) => {
        impl TryMul<$repr> for Fraction {
            type Output = $repr;

            fn try_mul(self, rhs: $repr) -> Option<Self::Output> {
                use num_integer::Integer;
                let numerator: $repr = self.numerator().try_into().ok()?;
                let denominator: $repr = self.denominator().try_into().ok()?;
                let (div, rem) = rhs.checked_mul(numerator)?.div_rem(&denominator);
                if rem == 0 { Some(div) } else { None }
            }
        }

        impl TryMul<Fraction> for $repr {
            type Output = $repr;

            fn try_mul(self, rhs: Fraction) -> Option<Self::Output> {
                rhs.try_mul(self)
            }
        }
    };
}

try_mul_integer!(u8);
try_mul_integer!(u16);
try_mul_integer!(u32);
try_mul_integer!(u64);
try_mul_integer!(u128);
try_mul_integer!(i8);
try_mul_integer!(i16);
try_mul_integer!(i32);
try_mul_integer!(i64);
try_mul_integer!(i128);

impl TryMul<f64> for Fraction {
    type Output = f64;

    fn try_mul(self, rhs: f64) -> Option<Self::Output> {
        Some(self * rhs)
    }
}

impl TryMul<f32> for Fraction {
    type Output = f32;

    fn try_mul(self, rhs: f32) -> Option<Self::Output> {
        Some(self * rhs)
    }
}

impl TryMul<Fraction> for f64 {
    type Output = f64;

    fn try_mul(self, rhs: Fraction) -> Option<Self::Output> {
        Some(self * rhs)
    }
}

impl TryMul<Fraction> for f32 {
    type Output = f32;

    fn try_mul(self, rhs: Fraction) -> Option<Self::Output> {
        Some(self * rhs)
    }
}

/// Trait representing multiplication that always succeeds, but that will round-to-nearest (upwards
/// on tie) if the result is not an integer.
pub trait MulRound<T> {
    type Output;

    /// Multiplies `self` by `rhs`. If the output is not an integer, applies rounding to nearest,
    /// with upwards rounding on tie.
    fn mul_round(self, rhs: T) -> Self::Output;
}

macro_rules! mul_round_unsigned_integer {
    ($repr:ty) => {
        impl MulRound<$repr> for Fraction {
            type Output = $repr;

            fn mul_round(self, rhs: $repr) -> Self::Output {
                let numerator = self.numerator() as $repr;
                let denominator = self.denominator() as $repr;
                let half = denominator >> 1;
                let (div, rem) = num_integer::div_rem(rhs * numerator, denominator);
                if rem > half { div + 1 } else { div }
            }
        }

        impl MulRound<Fraction> for $repr {
            type Output = $repr;

            fn mul_round(self, rhs: Fraction) -> Self::Output {
                rhs.mul_round(self)
            }
        }
    };
}

macro_rules! mul_round_signed_integer {
    ($repr:ty) => {
        impl MulRound<$repr> for Fraction {
            type Output = $repr;

            fn mul_round(self, rhs: $repr) -> Self::Output {
                use num_traits::ConstZero;
                let numerator = self.numerator() as $repr;
                let denominator = self.denominator() as $repr;
                let half = denominator >> 1;
                let (div, rem) = num_integer::div_rem(rhs * numerator, denominator);
                if rhs >= <$repr>::ZERO {
                    if rem > half { div + 1 } else { div }
                } else {
                    if rem < (-half) { div - 1 } else { div }
                }
            }
        }

        impl MulRound<Fraction> for $repr {
            type Output = $repr;

            fn mul_round(self, rhs: Fraction) -> Self::Output {
                rhs.mul_round(self)
            }
        }
    };
}

mul_round_unsigned_integer!(u8);
mul_round_unsigned_integer!(u16);
mul_round_unsigned_integer!(u32);
mul_round_unsigned_integer!(u64);
mul_round_unsigned_integer!(u128);
mul_round_signed_integer!(i8);
mul_round_signed_integer!(i16);
mul_round_signed_integer!(i32);
mul_round_signed_integer!(i64);
mul_round_signed_integer!(i128);

impl MulRound<f64> for Fraction {
    type Output = f64;

    fn mul_round(self, rhs: f64) -> Self::Output {
        (self * rhs).round()
    }
}

impl MulRound<Fraction> for f64 {
    type Output = f64;

    fn mul_round(self, rhs: Fraction) -> Self::Output {
        rhs.mul_round(self)
    }
}

impl MulRound<f32> for Fraction {
    type Output = f32;

    fn mul_round(self, rhs: f32) -> Self::Output {
        (self * rhs).round()
    }
}

impl MulRound<Fraction> for f32 {
    type Output = f32;

    fn mul_round(self, rhs: Fraction) -> Self::Output {
        rhs.mul_round(self)
    }
}

/// Trait representing multiplication that always succeeds, but that will round towards negative
/// infinity if the output is not an integer.
pub trait MulFloor<T> {
    type Output;

    /// Multiplies `self` by `rhs`. If the output is not an integer, rounds towards negative
    /// infinity.
    fn mul_floor(self, rhs: T) -> Self::Output;
}

macro_rules! mul_floor_integer {
    ($repr:ty) => {
        impl MulFloor<Fraction> for $repr {
            type Output = $repr;

            fn mul_floor(self, rhs: Fraction) -> Self::Output {
                let numerator = rhs.numerator() as Self;
                let denominator = rhs.denominator() as Self;
                num_integer::div_floor(self * numerator, denominator)
            }
        }

        impl MulFloor<$repr> for Fraction {
            type Output = $repr;

            fn mul_floor(self, rhs: $repr) -> Self::Output {
                rhs.mul_floor(self)
            }
        }
    };
}

mul_floor_integer!(u8);
mul_floor_integer!(u16);
mul_floor_integer!(u32);
mul_floor_integer!(u64);
mul_floor_integer!(u128);
mul_floor_integer!(i8);
mul_floor_integer!(i16);
mul_floor_integer!(i32);
mul_floor_integer!(i64);
mul_floor_integer!(i128);

impl MulFloor<Fraction> for f64 {
    type Output = f64;

    fn mul_floor(self, rhs: Fraction) -> Self::Output {
        (self * rhs).floor()
    }
}

impl MulFloor<f64> for Fraction {
    type Output = f64;

    fn mul_floor(self, rhs: f64) -> Self::Output {
        (self * rhs).floor()
    }
}

impl MulFloor<Fraction> for f32 {
    type Output = f32;

    fn mul_floor(self, rhs: Fraction) -> Self::Output {
        (self * rhs).floor()
    }
}

impl MulFloor<f32> for Fraction {
    type Output = f32;

    fn mul_floor(self, rhs: f32) -> Self::Output {
        (self * rhs).floor()
    }
}

/// Trait representing multiplication that always succeeds, but that will round towards positive
/// infinity if the output is not an integer.
pub trait MulCeil<T> {
    type Output;

    /// Multiplies `self` by `rhs`. If the output is not an integer, rounds towards positive
    /// infinity.
    fn mul_ceil(self, rhs: T) -> Self::Output;
}

macro_rules! mul_ceil_integer {
    ($repr:ty) => {
        impl MulCeil<Fraction> for $repr {
            type Output = $repr;

            fn mul_ceil(self, rhs: Fraction) -> Self::Output {
                let numerator = rhs.numerator() as Self;
                let denominator = rhs.denominator() as Self;
                num_integer::div_ceil(self * numerator, denominator)
            }
        }

        impl MulCeil<$repr> for Fraction {
            type Output = $repr;

            fn mul_ceil(self, rhs: $repr) -> Self::Output {
                rhs.mul_ceil(self)
            }
        }
    };
}

mul_ceil_integer!(u8);
mul_ceil_integer!(u16);
mul_ceil_integer!(u32);
mul_ceil_integer!(u64);
mul_ceil_integer!(u128);
mul_ceil_integer!(i8);
mul_ceil_integer!(i16);
mul_ceil_integer!(i32);
mul_ceil_integer!(i64);
mul_ceil_integer!(i128);

impl MulCeil<Fraction> for f64 {
    type Output = f64;

    fn mul_ceil(self, rhs: Fraction) -> Self::Output {
        (self * rhs).ceil()
    }
}

impl MulCeil<f64> for Fraction {
    type Output = f64;

    fn mul_ceil(self, rhs: f64) -> Self::Output {
        (self * rhs).ceil()
    }
}

impl MulCeil<Fraction> for f32 {
    type Output = f32;

    fn mul_ceil(self, rhs: Fraction) -> Self::Output {
        (self * rhs).ceil()
    }
}

impl MulCeil<f32> for Fraction {
    type Output = f32;

    fn mul_ceil(self, rhs: f32) -> Self::Output {
        (self * rhs).ceil()
    }
}
