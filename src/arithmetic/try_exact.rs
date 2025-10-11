//! Conversion operation that is equivalent to `TryFrom`/`TryInto` but also permits
//! float-to-integer conversions when the result is exact.

use core::fmt::Debug;

use num_traits::{ConstZero, Float, Zero};
use thiserror::Error;

pub trait TryFromExact<T>: Sized {
    type Error;

    fn try_from_exact(value: T) -> Result<Self, Self::Error>;
}

macro_rules! derive_from_try_from {
    ($from:ty, $into:ty) => {
        impl TryFromExact<$from> for $into {
            type Error = <$into as TryFrom<$from>>::Error;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                <$into>::try_from(value)
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
        derive_from_try_from!($from, i8);
        derive_from_try_from!($from, i16);
        derive_from_try_from!($from, i32);
        derive_from_try_from!($from, i64);
        derive_from_try_from!($from, i128);
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

pub trait TryIntoExact<T>: Sized {
    type Error;

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

                if !value.fract().is_zero() {
                    return Err(TryUnsignedFromFloatError::NonIntegerFloat { float: value });
                }

                if value.is_sign_negative() {
                    return Err(TryUnsignedFromFloatError::NegativeFloat { float: value });
                }

                let (mantissa, exponent, _) = value.integer_decode();
                let integer = if exponent > 0 {
                    mantissa as u128 * 2u128.pow(exponent as u32)
                } else if exponent < 0 {
                    mantissa as u128 / 2u128.pow(-exponent as u32)
                } else {
                    mantissa as u128
                };
                match integer.try_into() {
                    Ok(integer) => Ok(integer),
                    Err(_) => Err(TryUnsignedFromFloatError::OutOfBounds {
                        result: integer,
                        _marker: core::marker::PhantomData,
                    }),
                }
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
    #[error("result ({result}) outside of representable bounds for type {}", core::any::type_name::<Unsigned>())]
    OutOfBounds {
        result: u128,
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

                if !value.fract().is_zero() {
                    return Err(TrySignedFromFloatError::NonIntegerFloat { float: value });
                }

                let (mantissa, exponent, sign) = value.integer_decode();
                let integer = if exponent > 0 {
                    sign as i128 * mantissa as i128 * 2i128.pow(exponent as u32)
                } else if exponent < 0 {
                    sign as i128 * mantissa as i128 / 2i128.pow(-exponent as u32)
                } else {
                    sign as i128 * mantissa as i128
                };
                match integer.try_into() {
                    Ok(integer) => Ok(integer),
                    Err(_) => Err(TrySignedFromFloatError::OutOfBounds {
                        result: integer,
                        _marker: core::marker::PhantomData,
                    }),
                }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum TrySignedFromFloatError<Float, Signed>
where
    Float: Debug,
{
    #[error("float has non-integer part: {float:?}")]
    NonIntegerFloat { float: Float },
    #[error("float is negative: {float:?}")]
    NegativeFloat { float: Float },
    #[error("result ({result}) outside of representable bounds for type {}", core::any::type_name::<Signed>())]
    OutOfBounds {
        result: i128,
        _marker: core::marker::PhantomData<Signed>,
    },
}
