//! Implementation of flooring division.

use core::ops::Div;

/// Trait that represents flooring division of one value by another. Equivalent to `div_floor()` in
/// `num::Integer`, but also implemented for floating point values.
pub trait DivFloor {
    fn div_floor(&self, other: &Self) -> Self;
}

impl DivFloor for f32 {
    fn div_floor(&self, other: &Self) -> Self {
        self.div(other).floor()
    }
}

impl DivFloor for f64 {
    fn div_floor(&self, other: &Self) -> Self {
        self.div(other).floor()
    }
}

impl DivFloor for u8 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for u16 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for u32 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for u64 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for u128 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for i8 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for i16 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for i32 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for i64 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}

impl DivFloor for i128 {
    fn div_floor(&self, other: &Self) -> Self {
        <Self as num::Integer>::div_floor(self, other)
    }
}
