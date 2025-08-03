//! Implementation of the Global Positioning System (GPS) time scale, generally abbreviated as
//! GPST.

use crate::{
    Bdt, FromTimeScale, Glonasst, Gst, LeapSecondError, LocalTime, Qzsst, Tai, TaiTime, TimePoint,
    TimeScale, TryFromTimeScale, Tt, Unix, Utc,
    arithmetic::{FromUnit, Second, TimeRepresentation, TryFromExact, Unit},
    calendar::{Date, Month},
};

/// `GpsTime` is a time point that is expressed according to the GPS time scale.
pub type GpsTime<Representation, Period = Second> = TimePoint<Gpst, Representation, Period>;

/// The Global Positioning System (GPS) time scale is broadcast by GPS satellites. It is based on
/// internal atomic clocks that are synchronized with TAI. The signal is defined to be a constant
/// 19 seconds behind TAI.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gpst;

impl TimeScale for Gpst {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_generic_datetime(Date::new(1980, Month::January, 6).unwrap(), 0, 0, 19)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }

    fn epoch_local() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        Date::new(1980, Month::January, 6)
            .unwrap()
            .to_local_days()
            .into_unit()
            .try_cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl FromTimeScale<Bdt> for Gpst {}
impl FromTimeScale<Glonasst> for Gpst {}
impl FromTimeScale<Gst> for Gpst {}
impl FromTimeScale<Qzsst> for Gpst {}
impl FromTimeScale<Tai> for Gpst {}
impl FromTimeScale<Utc> for Gpst {}
impl FromTimeScale<Tt> for Gpst {}

impl TryFromTimeScale<Unix> for Gpst {
    type Error = LeapSecondError;

    fn try_from_time_scale<Representation, Period>(
        from: TimePoint<Unix, Representation, Period>,
    ) -> Result<TimePoint<Self, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<<Unix as TimeScale>::NativePeriod, <Unix as TimeScale>::NativeRepresentation>
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<<Unix as TimeScale>::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
    {
        let utc_time = Utc::try_from_time_scale(from)?;
        Ok(utc_time.into_time_scale())
    }
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    let tai = TaiTime::from_generic_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 32)
        .unwrap();
    let gpst = GpsTime::from_generic_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 13)
        .unwrap();
    assert_eq!(tai, gpst.into_time_scale());
}

/// Compares with some week numbers as computed using the LabSat GPS time calculator (found at
/// https://www.labsat.co.uk/index.php/en/gps-time-calculator).
#[test]
fn week_numbers() {
    use crate::{Seconds, UtcTime};
    let from_week = GpsTime::from_week_time(1625, Seconds::new(364379));
    let expected = UtcTime::from_datetime(2011, Month::March, 3, 5, 12, 44).unwrap();
    assert_eq!(from_week, expected.into_time_scale());

    let from_week = GpsTime::from_week_time(854, Seconds::new(845));
    let expected = UtcTime::from_datetime(1996, Month::May, 19, 0, 13, 54).unwrap();
    assert_eq!(from_week, expected.into_time_scale());

    let from_week = GpsTime::from_week_time(2378, Seconds::new(64617));
    let expected = UtcTime::from_datetime(2025, Month::August, 3, 17, 56, 39).unwrap();
    assert_eq!(from_week, expected.into_time_scale());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a GPS time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GpsTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a GPS time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GpsTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that all valid GPS time datetimes can be losslessly converted to and from
    /// the equivalent TAI time.
    #[kani::proof]
    fn datetime_tai_roundtrip() {
        let date: Date = kani::any();
        let year: i32 = date.year();
        let month: Month = date.month();
        let day: u8 = date.day();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1 = GpsTime::from_datetime(year, month, day, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.into_time_scale();
        let time2: GpsTime<_> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
