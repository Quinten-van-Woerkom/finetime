//! Implementation of the BeiDou Time (BDT) time scale.

use crate::{
    LeapSecondError, LocalTime, TaiTime, TerrestrialTimeScale, TimePoint, TimeScale,
    TryFromTimeScale, Unix, Utc,
    arithmetic::{FromUnit, Second, TimeRepresentation, TryFromExact, Unit},
    calendar::{Date, Month},
};

/// `BeiDouTime` is a time point that is expressed according to the BeiDou Time time
/// scale.
pub type BeiDouTime<Representation, Period = Second> = TimePoint<Bdt, Representation, Period>;

/// The BeiDou Time (BDT) time scale is broadcast by BeiDou satellites. It is a continuous time
/// scale, but may apply frequency adjustments to stay consistent with UTC (beyond the leap
/// seconds, which are not applied). It starts at 00:00:00 UTC, January 1, 2006.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bdt;

impl TimeScale for Bdt {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        Date::new(2006, Month::January, 1)
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

impl TerrestrialTimeScale for Bdt {
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_datetime(2006, Month::January, 1, 0, 0, 33)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Bdt {
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

/// Compares with a known timestamp as obtained from the definition of the BeiDou Time: the
/// epoch itself of the system.
#[test]
fn known_timestamps() {
    use crate::UtcTime;
    let utc = UtcTime::from_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    let bdt = BeiDouTime::from_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(utc, bdt.into_time_scale());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a BeiDou time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = BeiDouTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a BeiDou time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = BeiDouTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that all valid BeiDou time datetimes can be losslessly converted to and from
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
        let time1 = BeiDouTime::from_datetime(year, month, day, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.into_time_scale();
        let time2: BeiDouTime<_> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
