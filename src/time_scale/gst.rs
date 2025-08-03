//! Implementation of the Galileo System Time (GST) time scale.

use crate::{
    LeapSecondError, LocalTime, TaiTime, TerrestrialTimeScale, TimePoint, TimeScale,
    TryFromTimeScale, Unix, Utc,
    arithmetic::{FromUnit, Second, TimeRepresentation, TryFromExact, Unit},
    calendar::{Date, Month},
};

/// `GalileoTime` is a time point that is expressed according to the Galileo System Time time
/// scale.
pub type GalileoTime<Representation, Period = Second> = TimePoint<Gst, Representation, Period>;

/// The Galileo System Time (GST) time scale is broadcast by Galileo satellites. It is based on
/// internal atomic clocks that are synchronized with TAI. The signal is defined to be a constant
/// 19 seconds behind TAI.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gst;

impl TimeScale for Gst {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        Date::new(1999, Month::August, 22)
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

impl TerrestrialTimeScale for Gst {
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_generic_datetime(Date::new(1999, Month::August, 22).unwrap(), 0, 0, 19)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Gst {
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

/// Compares with a known timestamp as obtained from the definition of the Galileo System Time: the
/// epoch itself of the system.
#[test]
fn known_timestamps() {
    use crate::UtcTime;
    let utc = UtcTime::from_datetime(1999, Month::August, 21, 23, 59, 47).unwrap();
    let gst = GalileoTime::from_datetime(1999, Month::August, 22, 0, 0, 0).unwrap();
    assert_eq!(utc, gst.into_time_scale());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a Galileo time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GalileoTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a Galileo time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GalileoTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that all valid Galileo time datetimes can be losslessly converted to and from
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
        let time1 = GalileoTime::from_datetime(year, month, day, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.into_time_scale();
        let time2: GalileoTime<_> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
