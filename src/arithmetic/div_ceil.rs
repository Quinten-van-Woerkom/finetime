//! Implementation of ceiling division.

use core::ops::Div;

/// Trait that represents ceiling division of one value by another. Equivalent to `div_ceil()` in
/// `num_integer::Integer`, but also implemented for floating point values.
pub trait DivCeil {
    fn div_ceil(&self, other: &Self) -> Self;
}

impl DivCeil for f32 {
    fn div_ceil(&self, other: &Self) -> Self {
        self.div(other).ceil()
    }
}

impl DivCeil for f64 {
    fn div_ceil(&self, other: &Self) -> Self {
        self.div(other).ceil()
    }
}

impl DivCeil for u8 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for u16 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for u32 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for u64 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for u128 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for i8 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for i16 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for i32 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for i64 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}

impl DivCeil for i128 {
    fn div_ceil(&self, other: &Self) -> Self {
        <Self as num_integer::Integer>::div_ceil(self, other)
    }
}
