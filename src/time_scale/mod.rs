//! Implementation of timekeeping according to different time scales.

mod convert;
pub use convert::{FromTimeScale, IntoTimeScale};
mod datetime;
pub use datetime::{
    FromDateTime, FromFineDateTime, IntoDateTime, IntoFineDateTime, UniformDateTimeScale,
};

mod gpst;
pub use gpst::{GpsTime, Gpst};
mod gst;
pub use gst::{GalileoTime, Gst};
mod leap_seconds;
pub use leap_seconds::{
    FromLeapSecondDateTime, IntoLeapSecondDateTime, LeapSecondProvider,
    STATIC_LEAP_SECOND_PROVIDER, StaticLeapSecondProvider,
};
mod tai;
pub use tai::{Tai, TaiTime};
mod tcg;
pub use tcg::{Tcg, TcgTime};
mod tt;
pub use tt::{Tt, TtTime};
mod terrestrial_time;
pub use terrestrial_time::TerrestrialTime;
mod utc;
pub use utc::{Utc, UtcTime};

use crate::Date;

pub trait TimeScale {
    /// The full (English) name of a time scale.
    const NAME: &'static str;

    /// The abbreviated string used to represent this time scale.
    const ABBREVIATION: &'static str;

    /// Determines the epoch used to convert date-time of this time scale into the equivalent
    /// time-since-epoch `TimePoint` representation. For simplicity, epochs must fall on date
    /// boundaries.
    ///
    /// Note that this epoch does not bear any "real" significance: it is merely a representational
    /// choice. In principle, it may even be distinct from the "actual" epoch, if any is defined,
    /// for a time scale. For GPS, for example, the epoch is well-defined as 1980-01-06T00:00:00
    /// UTC, but it would not necessarily be wrong to use a different date here. In practice, of
    /// course, it is more convenient to choose the actual epoch where one is defined.
    const EPOCH: Date<i32>;
}
