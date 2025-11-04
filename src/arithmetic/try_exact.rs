//! Conversion operation that is equivalent to `TryFrom`/`TryInto` but also permits
//! float-to-integer conversions when the result is exact.

use core::fmt::Debug;

#[cfg(feature = "i256")]
use i256::{I256, U256};

use num_traits::{ConstZero, Float, Zero};
use thiserror::Error;

/// Extension of `TryFrom` that behaves exactly the same, but may also define exact float
/// conversions.
pub trait TryFromExact<T>: Sized {
    type Error;

    /// Tries to convert `value` into type `Self`. If this conversion may succeed without loss of
    /// information, returns `Ok(_)` with the converted value. If any information may be lost (even
    /// if it is only floating point rounding), returns `None`.
    ///
    /// The primarily reason to use this trait is when conversions from floats to integers are
    /// needed: these are not supported by the standard `TryFrom` implementations.
    fn try_from_exact(value: T) -> Result<Self, Self::Error>;
}

/// An `f64` may always be converted into itself. We do not implement a generic self-to-self
/// because that leads to problems with conflicting implementations for other generics.
impl TryFromExact<f64> for f64 {
    type Error = core::convert::Infallible;

    fn try_from_exact(value: f64) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

/// An `f32` may always be converted into itself. We do not implement a generic self-to-self
/// because that leads to problems with conflicting implementations for other generics.
impl TryFromExact<f32> for f32 {
    type Error = core::convert::Infallible;

    fn try_from_exact(value: f32) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

macro_rules! derive_from_try_from {
    ($from:ty, $into:ty) => {
        impl TryFromExact<$from> for $into {
            type Error = <$into as TryFrom<$from>>::Error;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                <$into>::try_from(value)
            }
        }

        #[cfg(kani)]
        paste::paste! {
            /// This proof harness ensures that none of the derived `TryFromExact` implementations
            /// ever result in undefined behaviour, panics, or arithmetic errors.
            #[allow(non_snake_case)]
            #[kani::proof]
            fn [<try_from_exact_ $into _from_ $from _infallible >]() {
                use crate::TryIntoExact;
                let from: $from = kani::any();
                let _result: Result<$into, _> = from.try_into_exact();
            }
        }
    };
}

macro_rules! derive_from_try_from_all_integers {
    ($from:ty) => {
        derive_from_try_from!($from, u8);
        derive_from_try_from!($from, u16);
        derive_from_try_from!($from, u32);
        derive_from_try_from!($from, u64);
        derive_from_try_from!($from, u128);
        #[cfg(feature = "i256")]
        derive_from_try_from!($from, U256);
        derive_from_try_from!($from, i8);
        derive_from_try_from!($from, i16);
        derive_from_try_from!($from, i32);
        derive_from_try_from!($from, i64);
        derive_from_try_from!($from, i128);
        #[cfg(feature = "i256")]
        derive_from_try_from!($from, I256);
    };
}

derive_from_try_from_all_integers!(u8);
derive_from_try_from_all_integers!(u16);
derive_from_try_from_all_integers!(u32);
derive_from_try_from_all_integers!(u64);
derive_from_try_from_all_integers!(u128);
derive_from_try_from_all_integers!(i8);
derive_from_try_from_all_integers!(i16);
derive_from_try_from_all_integers!(i32);
derive_from_try_from_all_integers!(i64);
derive_from_try_from_all_integers!(i128);

/// Trait representing the converse of `TryFromExact`. Similar to the standard `TryFrom` and
/// `TryInto` traits, it is advised not to implement this trait directly but rather to implement
/// `TryFromExact` and let `TryIntoExact` be derived.
pub trait TryIntoExact<T>: Sized {
    type Error;

    /// Tries to convert `self` into type `T`. If this conversion may succeed without loss of
    /// information, returns `Ok(_)` with the converted value. If any information may be lost (even
    /// if it is only floating point rounding), returns `None`.
    fn try_into_exact(self) -> Result<T, Self::Error>;
}

impl<T, U> TryIntoExact<T> for U
where
    T: TryFromExact<U>,
{
    type Error = T::Error;

    fn try_into_exact(self) -> Result<T, Self::Error> {
        T::try_from_exact(self)
    }
}

macro_rules! impl_unsigned_from_float {
    ($from:ty, $into:ty) => {
        impl TryFromExact<$from> for $into {
            type Error = TryUnsignedFromFloatError<$from, $into>;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                if value.is_zero() {
                    return Ok(<$into>::ZERO);
                }

                if value.is_infinite() {
                    return Err(TryUnsignedFromFloatError::Infinity);
                }

                if value.is_nan() {
                    return Err(TryUnsignedFromFloatError::Nan);
                }

                if value.is_sign_negative() {
                    return Err(TryUnsignedFromFloatError::NegativeFloat { float: value });
                }

                if !value.fract().is_zero() {
                    return Err(TryUnsignedFromFloatError::NonIntegerFloat { float: value });
                }

                let (mantissa, exponent, _) = value.integer_decode();
                let mantissa = mantissa as u128;
                let out_of_bounds = TryUnsignedFromFloatError::OutOfBounds {
                    result: value,
                    _marker: core::marker::PhantomData,
                };
                let integer = if exponent > 0 {
                    let factor = 2u128.checked_pow(exponent as u32).ok_or(out_of_bounds)?;
                    mantissa.checked_mul(factor).ok_or(out_of_bounds)?
                } else if exponent < 0 {
                    let factor = 2u128.checked_pow(-exponent as u32).ok_or(out_of_bounds)?;
                    mantissa.checked_div(factor).ok_or(out_of_bounds)?
                } else {
                    mantissa
                };
                integer.try_into().or(Err(out_of_bounds))
            }
        }

        #[cfg(kani)]
        paste::paste! {
            /// This proof harness ensures that none of the unsigned-from-float `TryFromExact`
            /// implementations ever result in undefined behaviour, panics, or arithmetic errors.
            #[kani::proof]
            fn [<try_from_exact_ $into _from_ $from _infallible >]() {
                use crate::TryIntoExact;
                let from: $from = kani::any();
                let _result: Result<$into, _> = from.try_into_exact();
            }
        }
    };
}

impl_unsigned_from_float!(f32, u8);
impl_unsigned_from_float!(f32, u16);
impl_unsigned_from_float!(f32, u32);
impl_unsigned_from_float!(f32, u64);
impl_unsigned_from_float!(f32, u128);
impl_unsigned_from_float!(f64, u8);
impl_unsigned_from_float!(f64, u16);
impl_unsigned_from_float!(f64, u32);
impl_unsigned_from_float!(f64, u64);
impl_unsigned_from_float!(f64, u128);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum TryUnsignedFromFloatError<Float, Unsigned>
where
    Float: Debug,
    Unsigned: Debug,
{
    #[error("float has non-integer part: {float:?}")]
    NonIntegerFloat { float: Float },
    #[error("float is negative: {float:?}")]
    NegativeFloat { float: Float },
    #[error("float has value of infinity")]
    Infinity,
    #[error("float is NaN")]
    Nan,
    #[error("result ({result}) outside of representable bounds for type {}", core::any::type_name::<Unsigned>())]
    OutOfBounds {
        result: Float,
        _marker: core::marker::PhantomData<Unsigned>,
    },
}

macro_rules! impl_signed_from_float {
    ($from:ty, $into:ty) => {
        impl TryFromExact<$from> for $into {
            type Error = TrySignedFromFloatError<$from, $into>;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                if value.is_zero() {
                    return Ok(<$into>::ZERO);
                }

                if value.is_infinite() {
                    return Err(TrySignedFromFloatError::Infinity);
                }

                if value.is_nan() {
                    return Err(TrySignedFromFloatError::Nan);
                }

                if !value.fract().is_zero() {
                    return Err(TrySignedFromFloatError::NonIntegerFloat { float: value });
                }

                let (mantissa, exponent, sign) = value.integer_decode();
                let signed_mantissa = sign as i128 * mantissa as i128;

                let out_of_bounds = TrySignedFromFloatError::OutOfBounds {
                    result: value,
                    _marker: core::marker::PhantomData,
                };
                let integer = if exponent > 0 {
                    let factor = 2i128.checked_pow(exponent as u32).ok_or(out_of_bounds)?;
                    signed_mantissa.checked_mul(factor).ok_or(out_of_bounds)?
                } else if exponent < 0 {
                    let factor = 2i128.checked_pow(-exponent as u32).ok_or(out_of_bounds)?;
                    signed_mantissa.checked_div(factor).ok_or(out_of_bounds)?
                } else {
                    signed_mantissa
                };
                integer.try_into().or(Err(out_of_bounds))
            }
        }

        #[cfg(kani)]
        paste::paste! {
            /// This proof harness ensures that none of the signed-from-float `TryFromExact`
            /// implementations ever result in undefined behaviour, panics, or arithmetic errors.
            #[kani::proof]
            fn [<try_from_exact_ $into _from_ $from _infallible >]() {
                use crate::TryIntoExact;
                let from: $from = kani::any();
                let _result: Result<$into, _> = from.try_into_exact();
            }
        }
    };
}

impl_signed_from_float!(f32, i8);
impl_signed_from_float!(f32, i16);
impl_signed_from_float!(f32, i32);
impl_signed_from_float!(f32, i64);
impl_signed_from_float!(f32, i128);
impl_signed_from_float!(f64, i8);
impl_signed_from_float!(f64, i16);
impl_signed_from_float!(f64, i32);
impl_signed_from_float!(f64, i64);
impl_signed_from_float!(f64, i128);

#[test]
fn try_from_exact_i8_from_f32_counterexample() {
    let from: f32 = -1.873381e+38f32;
    let _result: Result<i8, _> = from.try_into_exact();
}

#[test]
fn try_from_exact_i128_from_f64_counterexample() {
    let from: f64 = -9.745314e+288f64;
    let _result: Result<i128, _> = from.try_into_exact();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum TrySignedFromFloatError<Float, Signed>
where
    Float: Debug,
{
    #[error("float has non-integer part: {float:?}")]
    NonIntegerFloat { float: Float },
    #[error("float is negative: {float:?}")]
    NegativeFloat { float: Float },
    #[error("float has value of infinity")]
    Infinity,
    #[error("float is NaN")]
    Nan,
    #[error("result ({result}) outside of representable bounds for type {}", core::any::type_name::<Signed>())]
    OutOfBounds {
        result: Float,
        _marker: core::marker::PhantomData<Signed>,
    },
}

macro_rules! impl_float_from_integer {
    ($float:ty, $integer:ty) => {
        impl TryFromExact<$integer> for $float {
            type Error = TryFloatFromIntegerError<$float, $integer>;

            fn try_from_exact(value: $integer) -> Result<Self, Self::Error> {
                let float = value as $float;
                if float.is_infinite() || !float.fract().is_zero() || (float as $integer != value) {
                    Err(TryFloatFromIntegerError::InexactRepresentation {
                        integer: value,
                        _marker: core::marker::PhantomData,
                    })
                } else {
                    Ok(float)
                }
            }
        }

        #[cfg(kani)]
        paste::paste! {
            /// This proof harness ensures that none of the float-from-integer `TryFromExact`
            /// implementations ever result in undefined behaviour, panics, or arithmetic errors.
            #[kani::proof]
            fn [<try_from_exact_ $float _from_ $integer _infallible >]() {
                use crate::TryIntoExact;
                let from: $integer = kani::any();
                let _result: Result<$float, _> = from.try_into_exact();
            }
        }
    };
}

impl_float_from_integer!(f32, u8);
impl_float_from_integer!(f32, u16);
impl_float_from_integer!(f32, u32);
impl_float_from_integer!(f32, u64);
impl_float_from_integer!(f32, u128);
impl_float_from_integer!(f32, i8);
impl_float_from_integer!(f32, i16);
impl_float_from_integer!(f32, i32);
impl_float_from_integer!(f32, i64);
impl_float_from_integer!(f32, i128);
impl_float_from_integer!(f64, u8);
impl_float_from_integer!(f64, u16);
impl_float_from_integer!(f64, u32);
impl_float_from_integer!(f64, u64);
impl_float_from_integer!(f64, u128);
impl_float_from_integer!(f64, i8);
impl_float_from_integer!(f64, i16);
impl_float_from_integer!(f64, i32);
impl_float_from_integer!(f64, i64);
impl_float_from_integer!(f64, i128);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum TryFloatFromIntegerError<Float, Integer>
where
    Integer: Debug,
{
    #[error("integer ({integer:?}) cannot be represented exactly by float of type {}", core::any::type_name::<Float>())]
    InexactRepresentation {
        integer: Integer,
        _marker: core::marker::PhantomData<Float>,
    },
}

#[test]
fn try_from_exact_f32_from_u128_nan() {
    let integer = 0xffffffff_ffffffff_ffffffff_ffffffffu128;
    let _float: Result<f32, _> = integer.try_into_exact();
}
