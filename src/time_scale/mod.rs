//! Implementation of timekeeping according to different time scales.

mod datetime;
pub use datetime::DateTime;
mod tai;
pub use tai::{Tai, TaiTime};
