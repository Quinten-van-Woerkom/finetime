//! Representation of Galileo System Time (GST), which is broadcast by the Galileo constellation.

use crate::{
    Date, Duration, Month, Seconds, TerrestrialTime, TimePoint, UniformDateTimeScale,
    time_scale::{AbsoluteTimeScale, TimeScale},
    units::Second,
};

pub type GalileoTime<Representation = i64, Period = Second> =
    TimePoint<Gst, Representation, Period>;

/// Time scale representing the Galileo System Time (GST). GST has no leap seconds and increases
/// monotonically at a constant rate. It is distributed as part of the Galileo broadcast messages,
/// making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gst;

impl TimeScale for Gst {
    const NAME: &'static str = "Galileo System Time";

    const ABBREVIATION: &'static str = "GST";
}

impl AbsoluteTimeScale for Gst {
    const EPOCH: Date<i32> = match Date::from_historic_date(1999, Month::August, 22) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Gst {}

impl TerrestrialTime for Gst {
    type Representation = i8;
    type Period = Second;
    const TAI_OFFSET: Duration<Self::Representation, Self::Period> = Seconds::new(-19);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics". Note that that timestamp is given for GPS time: Galileo system time is always
/// aligned with GPS.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, TaiTime};
    let tai =
        TaiTime::<i64, Second>::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let gst = GalileoTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, gst.into_time_scale());
}
