#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
mod duration;
pub use duration::*;
mod errors;
pub use errors::*;
mod fraction;
pub use fraction::*;
mod units;
pub use units::*;
