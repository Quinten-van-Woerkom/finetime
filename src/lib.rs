#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
mod arithmetic;
pub use arithmetic::{
    Fraction, FractionalDigits, MulCeil, MulFloor, MulRound, TryFromExact, TryIntoExact, TryMul,
};
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
mod parse;
pub use parse::{DurationComponent, DurationDesignator};
mod time_point;
pub use time_point::TimePoint;
mod time_scale;
pub use time_scale::{DateTime, DateTimeRepresentation, Tai, TaiTime};
mod units;
pub use units::{ConvertUnit, TryConvertUnit, UnitRatio};
