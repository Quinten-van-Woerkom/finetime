//! Implementation of timekeeping according to different time scales.

mod convert;
pub use convert::{FromScale, IntoScale};
mod datetime;
pub use datetime::{
    ContinuousDateTimeScale, FromDateTime, FromFineDateTime, IntoDateTime, IntoFineDateTime,
};
mod tai;
pub use tai::{Tai, TaiTime};
