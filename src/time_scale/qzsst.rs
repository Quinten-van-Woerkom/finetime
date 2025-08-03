//! Implementation of the Japanse Quasi-Zenith Satellite System (QZSS) time scale, generally
//! abbreviated as QZSST.

use crate::{
    LeapSecondError, LocalTime, TaiTime, TerrestrialTimeScale, TimePoint, TimeScale,
    TryFromTimeScale, Unix, Utc,
    arithmetic::{FromUnit, Second, TimeRepresentation, TryFromExact, Unit},
    calendar::{Date, Month},
};

/// `QzssTime` is a time point that is expressed according to the QZSS time scale.
pub type QzssTime<Representation, Period = Second> = TimePoint<Qzsst, Representation, Period>;

/// The Quasi-Zenith Satellite System (QZSS) time scale is broadcast by QZSS satellites. It is
/// effectively defined in the same manner as GPS, for interoperability. However, it uses different
/// clocks for its realization. Hence, we define it separately, to make this distinction clear and
/// to permit type safety in cases where this difference is important (like when comparing the
/// realization accuracy of different time scales).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qzsst;

impl TimeScale for Qzsst {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        Date::new(1980, Month::January, 6)
            .unwrap()
            .to_local_days()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TerrestrialTimeScale for Qzsst {
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_datetime(1980, Month::January, 6, 0, 0, 19)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Qzsst {
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
    let tai = TaiTime::from_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let qzsst = QzssTime::from_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, qzsst.into_time_scale());
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
        let _ = QzssTime::from_datetime(year, month, day, hour, minute, second);
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
        let _ = QzssTime::from_gregorian_datetime(year, month, day, hour, minute, second);
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
        let time1 = QzssTime::from_datetime(year, month, day, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.into_time_scale();
        let time2: QzssTime<_> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
