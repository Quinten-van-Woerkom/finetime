//! Implementation of the Terrestrial Time (TT) time scale.

use crate::{
    Date, LeapSecondError, LocalTime, Month, TaiTime, TerrestrialTimeScale, TimePoint, TimeScale,
    TryFromTimeScale, Unix, Utc,
    arithmetic::{FromUnit, Milli, Second, TimeRepresentation, TryFromExact, Unit},
    duration::MilliSeconds,
};

/// A time point that is expressed in Terrestrial Time.
pub type TtTime<Representation, Period = Second> = TimePoint<Tt, Representation, Period>;

/// Terrestrial time is the proper time of a clock located on the Earth geoid. It is used in
/// astronomical tables, mostly. Effectively, it is little more than a constant offset from TAI.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tt;

impl TimeScale for Tt {
    type NativePeriod = Milli;

    type NativeRepresentation = i128;

    /// For practical reasons (conversion to and from TCG), it is convenient to set the TT epoch to
    /// 1977-01-01T00:00:32.184: at this time, TT and TCG match exactly (by definition).
    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        let date = Date::new(1977, Month::January, 1).unwrap();
        let epoch = date.to_local_days().into_unit() + MilliSeconds::new(32_184);
        epoch.try_cast().unwrap()
    }
}

impl TerrestrialTimeScale for Tt {
    /// Terrestrial time is exactly (by definition) 32.184 seconds ahead of TAI. This means that
    /// its epoch is precisely 1977-01-01T00:00:00 TAI.
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        let date = Date::new(1977, Month::January, 1).unwrap();
        TaiTime::from_generic_datetime(date, 0, 0, 0)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Tt {
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
    use crate::{Date, Month, TaiTime};
    let tai = TaiTime::from_datetime(2004, Month::May, 14, 16, 43, 32)
        .unwrap()
        .into_unit();
    let tt = TtTime::from_subsecond_generic_datetime(
        Date::new(2004, Month::May, 14).unwrap(),
        16,
        44,
        4,
        MilliSeconds::new(184),
    )
    .unwrap();
    assert_eq!(tai, tt.into_time_scale());
}

#[test]
fn date_decomposition() {
    let time = TtTime::from_datetime(2004, Month::May, 14, 16, 44, 4).unwrap();
    assert_eq!(time.gregorian_date().year(), 2004);
    assert_eq!(time.gregorian_date().month(), Month::May);
    assert_eq!(time.gregorian_date().day(), 14);
    assert_eq!(time.gregorian_date_hms().1, 16);
    assert_eq!(time.gregorian_date_hms().2, 44);
    assert_eq!(time.gregorian_date_hms().3, 4);
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a terrestrial time from a historic date and time stamp never
    /// panics. An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TT epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a terrestrial time from a Gregorian date and time stamp never
    /// panics. An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TT epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that all valid terrestrial time datetimes can be losslessly converted to and from
    /// the equivalent TAI time. An assumption is made on the input range because some dates result
    /// in a count of milliseconds from the TT epoch that is too large to store in an `i64`.
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
        let time1 = TtTime::from_datetime(year, month, day, hour, minute, second).unwrap();
        let tai: TaiTime<_, _> = time1.into_time_scale();
        let time2: TtTime<_, _> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
