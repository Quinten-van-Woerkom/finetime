//! Helper functions that are used for byte manipulation throughout the duration type. Primarily
//! useful in reducing boilerplate and making type implementations generic.

pub struct UnsignedInteger;
pub struct SignedInteger;
pub struct Float;

pub type SmallestNumberOfBytes<const N: usize, Type> =
    <() as SmallestNumberWithBytes<N, Type>>::Type;

pub trait SmallestNumberWithBytes<const N: usize, Sign> {
    type Type;
}

macro_rules! impl_smallest_unsigned {
    ($n:expr, $ty:ty) => {
        impl SmallestNumberWithBytes<$n, UnsignedInteger> for () {
            type Type = $ty;
        }
    };
}

impl_smallest_unsigned!(1, u8);
impl_smallest_unsigned!(2, u16);
impl_smallest_unsigned!(3, u32);
impl_smallest_unsigned!(4, u32);
impl_smallest_unsigned!(5, u64);
impl_smallest_unsigned!(6, u64);
impl_smallest_unsigned!(7, u64);
impl_smallest_unsigned!(8, u64);
impl_smallest_unsigned!(9, u128);
impl_smallest_unsigned!(10, u128);
impl_smallest_unsigned!(11, u128);
impl_smallest_unsigned!(12, u128);
impl_smallest_unsigned!(13, u128);
impl_smallest_unsigned!(14, u128);
impl_smallest_unsigned!(15, u128);
impl_smallest_unsigned!(16, u128);

macro_rules! impl_smallest_signed {
    ($n:expr, $ty:ty) => {
        impl SmallestNumberWithBytes<$n, SignedInteger> for () {
            type Type = $ty;
        }
    };
}

impl_smallest_signed!(1, i8);
impl_smallest_signed!(2, i16);
impl_smallest_signed!(3, i32);
impl_smallest_signed!(4, i32);
impl_smallest_signed!(5, i64);
impl_smallest_signed!(6, i64);
impl_smallest_signed!(7, i64);
impl_smallest_signed!(8, i64);
impl_smallest_signed!(9, i128);
impl_smallest_signed!(10, i128);
impl_smallest_signed!(11, i128);
impl_smallest_signed!(12, i128);
impl_smallest_signed!(13, i128);
impl_smallest_signed!(14, i128);
impl_smallest_signed!(15, i128);
impl_smallest_signed!(16, i128);

/// Checks that all byte sizes from 1 to 16 may be constructed.
#[test]
fn construct_arbitrary_byte_uint() {
    let _: SmallestNumberOfBytes<1, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<2, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<3, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<4, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<5, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<6, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<7, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<8, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<9, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<10, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<11, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<12, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<13, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<14, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<15, UnsignedInteger> = 0;
    let _: SmallestNumberOfBytes<16, UnsignedInteger> = 0;
}
