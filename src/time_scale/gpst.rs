//! Implementation of the time broadcast by the Global Positioning System (GPS).

use crate::{
    ContinuousDateTimeScale, Date, Duration, Month, Seconds, TerrestrialTime, TimePoint,
    time_scale::TimeScale, units::Second,
};

pub type GpsTime<Representation = i64, Period = Second> = TimePoint<Gpst, Representation, Period>;

/// Time scale representing the Global Positioning System Time (GPST). GPST has no leap seconds
/// and increases monotonically at a constant rate. It is distributed as part of the GPS broadcast
/// messages, making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gpst;

impl TimeScale for Gpst {
    const NAME: &'static str = "Global Positioning System Time";

    const ABBREVIATION: &'static str = "GPST";

    const EPOCH: Date<i32> = match Date::from_gregorian_date(1980, Month::January, 6) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl ContinuousDateTimeScale for Gpst {}

impl TerrestrialTime for Gpst {
    type Representation = i8;
    type Period = Second;
    const TAI_OFFSET: Duration<Self::Representation, Self::Period> = Seconds::new(-19);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    use crate::{IntoScale, TaiTime};
    let tai =
        TaiTime::<i64, Second>::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let gpst = GpsTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, gpst.into_scale());
}
