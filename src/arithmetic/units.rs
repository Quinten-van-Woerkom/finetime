//! Definitions of the different units that may be used to express `Duration`s. In essence, these
//! types are little more than labels that are associated with a given ratio to SI seconds, as may
//! be used to convert between arbitrary time periods.

use crate::arithmetic::Fraction;

/// Trait used to describe a time unit. Such units are always defined as an exact ratio to SI
/// seconds. Based on this ratio, conversions to other units are defined.
///
/// Units that are larger than 1 second (e.g., a minute) shall have a `RATIO` that is larger than
/// one. Smaller units shall have a smaller `RATIO` value.
pub trait Unit {
    const RATIO: Fraction;
}

/// Time unit that is described as an exact ratio with respect to seconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LiteralRatio<const NUMERATOR: i128, const DENOMINATOR: i128 = 1> {}

impl<const NUMERATOR: i128, const DENOMINATOR: i128> Unit for LiteralRatio<NUMERATOR, DENOMINATOR> {
    const RATIO: Fraction = Fraction::new(NUMERATOR, DENOMINATOR);
}

/// This trait is used as helper trait to compute the conversion ratio between two units. It is
/// separate from the `UnitConversion` trait because that trait depends on whether the
/// underlying representation can store fractional results: the actual conversion ratio shall
/// always remain the same, so is encoded separately. This allows it to be used in functions that
/// dynamically test whether a conversion with a given representation succeed exactly (like
/// `Duration::try_convert`).
pub trait ConversionRatio<To: Unit>: Unit {
    const CONVERSION_RATIO: Fraction;
}

impl<From: Unit, To: Unit> ConversionRatio<To> for From {
    const CONVERSION_RATIO: Fraction = From::RATIO.divide_by(&To::RATIO);
}

/// This trait indicates whether it is possible to convert a `Duration` or `TimePoint` with the
/// given underlying representation from units of `From` to `To`. It is advised not to implement
/// this trait directly, but rather to implement `FromUnit`. This trait will then be derived
/// automatically.
pub trait IntoUnit<To: Unit, Representation>: ConversionRatio<To> {}

/// This trait indicates whether it is possible to convert a `Duration` or `TimePoint` with the
/// given underlying representation from units of `From` to `To`. It has a similar relationship to
/// `IntoUnit` as `From` has to `Into`.
///
/// It is advised to always implement `FromUnit`: `IntoUnit` is derived automatically.
pub trait FromUnit<From: Unit, Representation>: ConversionRatio<From> {}

impl<From: Unit, To: Unit, Representation> IntoUnit<To, Representation> for From where
    To: FromUnit<From, Representation>
{
}

/// Floating point types support conversions to and from all unit ratios, because they do not lose
/// significant precision when multiplying with non-integer values.
impl<From: Unit, To: Unit> FromUnit<From, f32> for To {}

/// Floating point types support conversions to and from all unit ratios, because they do not lose
/// significant precision when multiplying with non-integer values.
impl<From: Unit, To: Unit> FromUnit<From, f64> for To {}

// For all integers, a conversion from a unit to itself is still valid.
impl<R: Unit> FromUnit<R, u8> for R {}
impl<R: Unit> FromUnit<R, u16> for R {}
impl<R: Unit> FromUnit<R, u32> for R {}
impl<R: Unit> FromUnit<R, u64> for R {}
impl<R: Unit> FromUnit<R, u128> for R {}
impl<R: Unit> FromUnit<R, i8> for R {}
impl<R: Unit> FromUnit<R, i16> for R {}
impl<R: Unit> FromUnit<R, i32> for R {}
impl<R: Unit> FromUnit<R, i64> for R {}
impl<R: Unit> FromUnit<R, i128> for R {}

macro_rules! is_valid_conversion {
    ($type:ty, $from:ty, $to:ty) => {
        impl FromUnit<$from, $type> for $to {}
    };
}

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
pub type SecondsPerDay = LiteralRatio<86400>;
pub type SecondsPerWeek = LiteralRatio<604800>;
/// The number of seconds in 1/12 the average Gregorian year.
pub type SecondsPerMonth = LiteralRatio<2629746>;
/// The number of seconds in an average Gregorian year.
pub type SecondsPerYear = LiteralRatio<31556952>;

// Conversions specific to time units
valid_integer_conversions!(SecondsPerMinute => Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerHour => SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerDay => SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerWeek => SecondsPerDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerMonth => SecondsPerWeek, SecondsPerDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);
valid_integer_conversions!(SecondsPerYear => SecondsPerMonth, SecondsPerWeek, SecondsPerDay, SecondsPerHour, SecondsPerMinute, Second, Deci, Centi, Milli, Micro, Nano, Pico, Femto, Atto);

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
