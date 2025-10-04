#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
mod calendar;
pub use calendar::{
    Date, GregorianDate, HistoricDate, JulianDate, JulianDay, ModifiedJulianDate, Month, WeekDay,
};
mod duration;
pub use duration::{
    AttoSeconds, Days, Duration, FemtoSeconds, HalfDays, Hours, MicroSeconds, MilliSeconds,
    Minutes, Months, NanoSeconds, PicoSeconds, Seconds, Weeks, Years,
};
pub mod errors;
mod fraction;
pub use fraction::{Fraction, MulCeil, MulFloor, MulRound, TryMul};
mod units;
pub use units::{Convert, TryConvert, UnitRatio};
