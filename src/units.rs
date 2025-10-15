//! Definitions of the different units that may be used to express `Duration`s. In essence, these
//! types are little more than labels that are associated with a given ratio to SI seconds, as may
//! be used to convert between arbitrary time periods.

use crate::{Fraction, TryMul};

/// Trait representing a lossless conversion from one unit to another. Note that the underlying
/// value representation stays the same. For floating point representations, floating point
/// rounding is permitted.
pub trait ConvertUnit<From, Into>
where
    From: ?Sized,
    Into: ?Sized,
{
    /// Converts from one unit into another. Shall only be used for exact conversions, without
    /// rounding error. Floating point errors are permitted.
    fn convert(self) -> Self;
}

macro_rules! impl_identity_conversion {
    ($repr:ty) => {
        impl<T> ConvertUnit<T, T> for $repr
        where
            T: UnitRatio,
        {
            fn convert(self) -> Self {
                self
            }
        }
    };
}

impl_identity_conversion!(u8);
impl_identity_conversion!(u16);
impl_identity_conversion!(u32);
impl_identity_conversion!(u64);
impl_identity_conversion!(u128);
impl_identity_conversion!(i8);
impl_identity_conversion!(i16);
impl_identity_conversion!(i32);
impl_identity_conversion!(i64);
impl_identity_conversion!(i128);

impl<From, Into> ConvertUnit<From, Into> for f64
where
    From: UnitRatio + ?Sized,
    Into: UnitRatio + ?Sized,
{
    fn convert(self) -> Self {
        let combined_ratio = From::FRACTION.divide_by(&Into::FRACTION);
        combined_ratio * self
    }
}

impl<From, Into> ConvertUnit<From, Into> for f32
where
    From: UnitRatio + ?Sized,
    Into: UnitRatio + ?Sized,
{
    fn convert(self) -> Self {
        let combined_ratio = From::FRACTION.divide_by(&Into::FRACTION);
        combined_ratio * self
    }
}

/// Trait representing a fallible conversion from one unit to another, failing if the requested
/// conversion cannot be computed losslessly. For floating point representations, this conversion
/// shall always succeed.
pub trait TryConvertUnit<From, Into>: Sized
where
    From: ?Sized,
    Into: ?Sized,
{
    /// Tries to convert from one unit into another. If the conversion would result in significant
    /// (non floating point) rounding error, returns `None`.
    fn try_convert(self) -> Option<Self>;
}

impl<T, From, Into> TryConvertUnit<From, Into> for T
where
    T: TryMul<Fraction, Output = T>,
    From: UnitRatio + ?Sized,
    Into: UnitRatio + ?Sized,
{
    fn try_convert(self) -> Option<Self> {
        let combined_ratio = From::FRACTION.divide_by(&Into::FRACTION);
        self.try_mul(combined_ratio)
    }
}

/// Trait representing the fact that something is a unit ratio.
pub trait UnitRatio {
    const FRACTION: Fraction;
}

/// Unit that is described as an exact ratio with respect to unity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralRatio<const NUMERATOR: u64, const DENOMINATOR: u64 = 1> {}

impl<const NUMERATOR: u64, const DENOMINATOR: u64> UnitRatio
    for LiteralRatio<NUMERATOR, DENOMINATOR>
{
    const FRACTION: Fraction = Fraction::new(NUMERATOR, DENOMINATOR);
}

macro_rules! valid_integer_conversions {
    (
        $from:ty => $( $to:ty ),+ $(,)?
    ) => {
        $(
            valid_integer_conversion!(u8, $from, $to);
            valid_integer_conversion!(u16, $from, $to);
            valid_integer_conversion!(u32, $from, $to);
            valid_integer_conversion!(u64, $from, $to);
            valid_integer_conversion!(u128, $from, $to);
            valid_integer_conversion!(i8, $from, $to);
            valid_integer_conversion!(i16, $from, $to);
            valid_integer_conversion!(i32, $from, $to);
            valid_integer_conversion!(i64, $from, $to);
            valid_integer_conversion!(i128, $from, $to);
        )+
    };
}

macro_rules! valid_integer_conversion {
    ($repr:ty, $from:ty, $into:ty) => {
        impl ConvertUnit<$from, $into> for $repr {
            fn convert(self) -> Self {
                let combined_ratio = <$from>::FRACTION.divide_by(&<$into>::FRACTION);
                // For any conversion ratio that is lossless, this division will not truncate.
                let factor = combined_ratio.numerator() / combined_ratio.denominator();
                self * (factor as Self)
            }
        }
    };
}

// SI unit qualifiers
pub type Atto = LiteralRatio<1, 1_000_000_000_000_000_000>;
pub type Femto = LiteralRatio<1, 1_000_000_000_000_000>;
pub type Pico = LiteralRatio<1, 1_000_000_000_000>;
pub type Nano = LiteralRatio<1, 1_000_000_000>;
pub type Micro = LiteralRatio<1, 1_000_000>;
pub type Milli = LiteralRatio<1, 1_000>;
pub type Centi = LiteralRatio<1, 100>;
pub type Deci = LiteralRatio<1, 10>;
pub type Deca = LiteralRatio<10>;
pub type Hecto = LiteralRatio<100>;
pub type Kilo = LiteralRatio<1_000>;
pub type Mega = LiteralRatio<1_000_000>;
pub type Giga = LiteralRatio<1_000_000_000>;
pub type Tera = LiteralRatio<1_000_000_000_000>;
pub type Peta = LiteralRatio<1_000_000_000_000_000>;
pub type Exa = LiteralRatio<1_000_000_000_000_000_000>;

// Conversions between regular SI units
valid_integer_conversions!(Femto => Atto);
valid_integer_conversions!(Pico => Femto, Atto);
valid_integer_conversions!(Nano => Pico, Femto, Atto);
valid_integer_conversions!(Micro => Nano, Pico, Femto, Atto);
valid_integer_conversions!(Milli => Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Centi => Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Deci => Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Second => Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Deca => Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Hecto => Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Kilo => Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Mega => Kilo, Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Giga => Mega, Kilo, Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Tera => Mega, Kilo, Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Peta => Tera, Giga, Mega, Kilo, Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Exa => Peta, Tera, Giga, Mega, Kilo, Hecto, Deca, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);

// Time unit qualifiers
pub type Second = LiteralRatio<1>;
pub type SecondsPerMinute = LiteralRatio<60>;
pub type SecondsPerHour = LiteralRatio<3600>;
/// Represents the number of seconds in half a day. Rather arbitrary "unit ratio", but turns out to
/// be useful in representing Julian days and modified Julian days.
pub type SecondsPerHalfDay = LiteralRatio<43200>;
pub type SecondsPerDay = LiteralRatio<86400>;
pub type SecondsPerWeek = LiteralRatio<604800>;
/// The number of seconds in 1/12 the average Gregorian year.
pub type SecondsPerMonth = LiteralRatio<2629746>;
/// The number of seconds in an average Gregorian year.
pub type SecondsPerYear = LiteralRatio<31556952>;

// Conversions specific to time units
valid_integer_conversions!(SecondsPerMinute => Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerHour => SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerHalfDay => SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerDay => SecondsPerHalfDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerWeek => SecondsPerDay, SecondsPerHalfDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerMonth => SecondsPerWeek, SecondsPerDay, SecondsPerHalfDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerYear => SecondsPerMonth, SecondsPerWeek, SecondsPerDay, SecondsPerHalfDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);

// Binary fractions of X bytes
pub type BinaryFraction1 = LiteralRatio<1, 0x100>;
pub type BinaryFraction2 = LiteralRatio<1, 0x10000>;
pub type BinaryFraction3 = LiteralRatio<1, 0x1000000>;
pub type BinaryFraction4 = LiteralRatio<1, 0x100000000>;
pub type BinaryFraction5 = LiteralRatio<1, 0x10000000000>;
pub type BinaryFraction6 = LiteralRatio<1, 0x1000000000000>;

// Conversions for binary fractions
valid_integer_conversions!(BinaryFraction5 => BinaryFraction6);
valid_integer_conversions!(BinaryFraction4 => BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(BinaryFraction3 => BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(BinaryFraction2 => BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(BinaryFraction1 => BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Second => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Deca => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Hecto => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Kilo => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Mega => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Giga => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Tera => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Peta => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Exa => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerMinute => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerHour => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerDay => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerWeek => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerMonth => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(SecondsPerYear => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
