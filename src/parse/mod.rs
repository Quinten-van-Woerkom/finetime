//! Implementation of parsing functionality. Largely separate from the actual time logic itself, so
//! kept in a separate module for isolation.
//!
//! Primarily, a subset of ISO 8601 is supported.

mod duration;
mod historic_date;
pub use duration::{DurationComponent, DurationDesignator};
mod decimal;
pub(crate) use decimal::DecimalNumber;
mod time_of_day;
pub(crate) use time_of_day::TimeOfDay;
mod time_point;
