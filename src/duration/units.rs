//! Definitions of the different units that may be used to express `Duration`s. In essence, these
//! types are little more than labels that are associated with a given ratio to SI seconds, as may
//! be used to convert between arbitrary time periods.

use crate::duration::fraction::Fraction;

/// Trait used to describe a time unit. Such units are always defined as an exact ratio to SI
/// seconds. Based on this ratio, conversions to other units are defined.
///
/// Units that are larger than 1 second (e.g., a minute) shall have a `RATIO` that is larger than
/// one. Smaller units shall have a smaller `RATIO` value.
pub trait Ratio {
    const RATIO: Fraction;
}

/// Time unit that is described as an exact ratio with respect to seconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LiteralRatio<const NUMERATOR: u64, const DENOMINATOR: u64 = 1> {}

impl<const NUMERATOR: u64, const DENOMINATOR: u64> Ratio for LiteralRatio<NUMERATOR, DENOMINATOR> {
    const RATIO: Fraction = Fraction::new(NUMERATOR, DENOMINATOR);
}

pub trait ConversionRatio<From: Ratio, To: Ratio> {
    const CONVERSION_RATIO: Fraction;
}

impl<From: Ratio, To: Ratio> ConversionRatio<From, To> for () {
    const CONVERSION_RATIO: Fraction = From::RATIO.divide_by(&To::RATIO);
}

pub trait IsValidConversion<Representation, From: Ratio, To: Ratio>:
    ConversionRatio<From, To>
{
}

/// Floating point types support conversions to and from all unit ratios, because they do not lose
/// significant precision when multiplying with non-integer values.
impl<From: Ratio, To: Ratio> IsValidConversion<f32, From, To> for () {}

/// Floating point types support conversions to and from all unit ratios, because they do not lose
/// significant precision when multiplying with non-integer values.
impl<From: Ratio, To: Ratio> IsValidConversion<f64, From, To> for () {}

// For all integers, a conversion from a unit to itself is still valid.
impl<R: Ratio> IsValidConversion<u8, R, R> for () {}
impl<R: Ratio> IsValidConversion<u16, R, R> for () {}
impl<R: Ratio> IsValidConversion<u32, R, R> for () {}
impl<R: Ratio> IsValidConversion<u64, R, R> for () {}
impl<R: Ratio> IsValidConversion<u128, R, R> for () {}
impl<R: Ratio> IsValidConversion<i8, R, R> for () {}
impl<R: Ratio> IsValidConversion<i16, R, R> for () {}
impl<R: Ratio> IsValidConversion<i32, R, R> for () {}
impl<R: Ratio> IsValidConversion<i64, R, R> for () {}
impl<R: Ratio> IsValidConversion<i128, R, R> for () {}

#[macro_export]
macro_rules! is_valid_conversion {
    ($type:ty, $from:ty, $to:ty) => {
        impl IsValidConversion<$type, $from, $to> for () {}
    };
}

#[macro_export]
macro_rules! is_valid_integer_conversion {
    ($from:ty, $to:ty) => {
        is_valid_conversion!(u8, $from, $to);
        is_valid_conversion!(u16, $from, $to);
        is_valid_conversion!(u32, $from, $to);
        is_valid_conversion!(u64, $from, $to);
        is_valid_conversion!(u128, $from, $to);
        is_valid_conversion!(i8, $from, $to);
        is_valid_conversion!(i16, $from, $to);
        is_valid_conversion!(i32, $from, $to);
        is_valid_conversion!(i64, $from, $to);
        is_valid_conversion!(i128, $from, $to);
    };
}

#[macro_export]
macro_rules! valid_integer_conversions {
    (
        $from:ty => $( $to:ty ),+ $(,)?
    ) => {
        $(
            is_valid_integer_conversion!($from, $to);
        )+
    }
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
valid_integer_conversions!(LiteralRatio<1> => Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Deca => LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Hecto => Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Kilo => Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Mega => Kilo, Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Giga => Mega, Kilo, Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Tera => Mega, Kilo, Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Peta => Tera, Giga, Mega, Kilo, Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(Exa => Peta, Tera, Giga, Mega, Kilo, Hecto, Deca, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);

// Time unit qualifiers
pub type SecondsPerMinute = LiteralRatio<60>;
pub type SecondsPerHour = LiteralRatio<3600>;
pub type SecondsPerDay = LiteralRatio<86400>;
pub type SecondsPerWeek = LiteralRatio<604800>;
/// The number of seconds in 1/12 the average Gregorian year.
pub type SecondsPerMonth = LiteralRatio<2629746>;
/// The number of seconds in an average Gregorian year.
pub type SecondsPerYear = LiteralRatio<31556952>;

// Conversions specific to time units
valid_integer_conversions!(SecondsPerMinute => LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerHour => SecondsPerMinute, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerDay => SecondsPerHour, SecondsPerMinute, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerWeek => SecondsPerDay, SecondsPerHour, SecondsPerMinute, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerMonth => SecondsPerWeek, SecondsPerDay, SecondsPerHour, SecondsPerMinute, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerYear => SecondsPerMonth, SecondsPerWeek, SecondsPerDay, SecondsPerHour, SecondsPerMinute, LiteralRatio<1>, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);

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
valid_integer_conversions!(LiteralRatio<1> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Deca => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Hecto => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Kilo => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Mega => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Giga => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Tera => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Peta => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(Exa => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<60> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<3600> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<86400> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<604800> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<2629746> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
valid_integer_conversions!(LiteralRatio<31556952> => BinaryFraction1, BinaryFraction2, BinaryFraction3, BinaryFraction4, BinaryFraction5, BinaryFraction6);
