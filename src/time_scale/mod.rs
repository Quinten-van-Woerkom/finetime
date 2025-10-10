//! Implementation of timekeeping according to different time scales.

mod datetime;
pub use datetime::{DateTime, DateTimeRepresentation};
mod tai;
pub use tai::{Tai, TaiTime};
