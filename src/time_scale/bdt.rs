//! Representation of BeiDou Time (BDT), which is broadcast by the BeiDou constellation.

use crate::{
    Date, Duration, Month, Seconds, TerrestrialTime, TimePoint, UniformDateTimeScale,
    time_scale::{AbsoluteTimeScale, TimeScale},
    units::Second,
};

pub type BeiDouTime<Representation = i64, Period = Second> = TimePoint<Bdt, Representation, Period>;

/// Time scale representing the BeiDou Time (BDT). BDT has no leap seconds and increases
/// monotonically at a constant rate. It is distributed as part of the BeiDou broadcast messages,
/// making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bdt;

impl TimeScale for Bdt {
    const NAME: &'static str = "BeiDou Time";

    const ABBREVIATION: &'static str = "BDT";
}

impl AbsoluteTimeScale for Bdt {
    const EPOCH: Date<i32> = match Date::from_historic_date(2006, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Bdt {}

impl TerrestrialTime for Bdt {
    type Representation = i8;
    type Period = Second;
    const TAI_OFFSET: Duration<Self::Representation, Self::Period> = Seconds::new(-33);
}

/// Compares with a known timestamp as obtained from the definition of the BeiDou Time: the
/// epoch itself of the system.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, UtcTime};
    let utc = UtcTime::from_historic_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    let bdt = BeiDouTime::from_historic_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(utc, bdt.into_time_scale());
}
