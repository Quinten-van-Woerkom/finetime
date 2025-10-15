//! Representation of Quasi-Zenith Satellite System Time (QZSST), which is broadcast by the
//! Quasi-Zenith Satellite System constellation.

use crate::{
    Date, Duration, Month, Seconds, TerrestrialTime, TimePoint, UniformDateTimeScale,
    time_scale::TimeScale, units::Second,
};

pub type QzssTime<Representation = i64, Period = Second> = TimePoint<Qzsst, Representation, Period>;

/// Time scale representing the Quasi-Zenith Satellite System Time (QZSST). QZSST has no leap
/// seconds and increases monotonically at a constant rate. It is distributed as part of the QZSST
/// broadcast messages, making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qzsst;

impl TimeScale for Qzsst {
    const NAME: &'static str = "Quasi-Zenith Satellite System Time";

    const ABBREVIATION: &'static str = "QZSST";

    const EPOCH: Date<i32> = match Date::from_gregorian_date(1999, Month::August, 22) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Qzsst {}

impl TerrestrialTime for Qzsst {
    type Representation = i8;
    type Period = Second;
    const TAI_OFFSET: Duration<Self::Representation, Self::Period> = Seconds::new(-19);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics". Note that that timestamp is given for GPS time: QZSS time is always aligned
/// with GPS.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, TaiTime};
    let tai =
        TaiTime::<i64, Second>::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let qzsst = QzssTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, qzsst.into_time_scale());
}
