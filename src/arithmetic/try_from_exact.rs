//! Implementation of an exact but fallible conversion. Similar to `TryFrom`, but also permits
//! conversions like `i64` to `f64` - but only if the resulting value is exactly representable in
//! the target type.

pub trait TryFromExact<T>: Sized {
    type Error;

    fn try_from_exact(value: T) -> Result<Self, Self::Error>;
}

pub trait TryIntoExact<T>: Sized {
    type Error;

    fn try_into_exact(self) -> Result<T, Self::Error>;
}

impl<To, From> TryIntoExact<To> for From
where
    To: TryFromExact<From>,
{
    type Error = <To as TryFromExact<From>>::Error;

    fn try_into_exact(self) -> Result<To, Self::Error> {
        To::try_from_exact(self)
    }
}

macro_rules! impl_try_from_exact_infallible {
    ($from:ty => $($int:ty),*) => {
        $(impl TryFromExact<$from> for $int {
            type Error = core::convert::Infallible;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                Ok(value.into())
            }
        })*
    };
}

impl_try_from_exact_infallible!(i128 => i128);
impl_try_from_exact_infallible!(i64 => i64, i128);
impl_try_from_exact_infallible!(i32 => i32, i64, i128, f64);
impl_try_from_exact_infallible!(i16 => i16, i32, i64, i128, f32, f64);
impl_try_from_exact_infallible!(i8 => i8, i16, i32, i64, i128, f32, f64);

impl_try_from_exact_infallible!(u128 => u128);
impl_try_from_exact_infallible!(u64 => u64, u128, i128);
impl_try_from_exact_infallible!(u32 => u32, u64, u128, i64, i128, f64);
impl_try_from_exact_infallible!(u16 => u16, u32, u64, u128, i32, i64, i128, f32, f64);
impl_try_from_exact_infallible!(u8 => u8, u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

impl_try_from_exact_infallible!(f32 => f32, f64);
impl_try_from_exact_infallible!(f64 => f64);

macro_rules! impl_try_from_exact_integer_fallible {
    ($from:ty => $($int:ty),*) => {
        $(impl TryFromExact<$from> for $int {
            type Error = <Self as TryFrom<$from>>::Error;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                Self::try_from(value)
            }
        })*
    };
}

impl_try_from_exact_integer_fallible!(u128 => u64, u32, u16, u8, i128, i64, i32, i16, i8);
impl_try_from_exact_integer_fallible!(u64 => u32, u16, u8, i64, i32, i16, i8);
impl_try_from_exact_integer_fallible!(u32 => u16, u8, i32, i16, i8);
impl_try_from_exact_integer_fallible!(u16 => u8, i16, i8);
impl_try_from_exact_integer_fallible!(u8 => i8);
impl_try_from_exact_integer_fallible!(i128 => u128, u64, u32, u16, u8, i64, i32, i16, i8);
impl_try_from_exact_integer_fallible!(i64 => u128, u64, u32, u16, u8, i32, i16, i8);
impl_try_from_exact_integer_fallible!(i32 => u128, u64, u32, u16, u8, i16, i8);
impl_try_from_exact_integer_fallible!(i16 => u128, u64, u32, u16, u8, i8);
impl_try_from_exact_integer_fallible!(i8 => u128, u64, u32, u16, u8);

macro_rules! impl_try_from_exact_float {
    ($from:ty => $($int:ty),*) => {
        $(impl TryFromExact<$from> for $int {
            type Error = IrrepresentableFloat<$from>;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                let integer = value as $int;
                if (integer as $from) == value {
                    Ok(integer)
                } else {
                    Err(IrrepresentableFloat(value))
                }
            }
        })*
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IrrepresentableFloat<Float>(Float);

impl_try_from_exact_float!(f64 => i128, i64, i32, i16, i8, u128, u64, u32, u16, u8);
impl_try_from_exact_float!(f32 => i128, i64, i32, i16, i8, u128, u64, u32, u16, u8);

macro_rules! impl_try_from_exact_integer {
    ($from:ty => $($float:ty),*) => {
        $(impl TryFromExact<$from> for $float {
            type Error = IrrepresentableInteger<$from>;

            fn try_from_exact(value: $from) -> Result<Self, Self::Error> {
                let integer = value as $float;
                if (integer as $from) == value {
                    Ok(integer)
                } else {
                    Err(IrrepresentableInteger(value))
                }
            }
        })*
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IrrepresentableInteger<Integer>(Integer);

impl_try_from_exact_integer!(i128 => f32, f64);
impl_try_from_exact_integer!(i64 => f32, f64);
impl_try_from_exact_integer!(i32 => f32);
impl_try_from_exact_integer!(u128 => f32, f64);
impl_try_from_exact_integer!(u64 => f32, f64);
impl_try_from_exact_integer!(u32 => f32);
