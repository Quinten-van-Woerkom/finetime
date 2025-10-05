//! Implementation of parsing functionality. Largely separate from the actual time logic itself, so
//! kept in a separate module for isolation.
//!
//! Primarily, a subset of ISO 8601 is supported.

mod historic_date;
mod duration;
pub use duration::{DurationComponent, DurationDesignator};
mod number;
pub(crate) use number::Number;
