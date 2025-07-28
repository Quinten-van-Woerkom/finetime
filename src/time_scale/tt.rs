//! Implementation of the Terrestrial Time (TT) time scale.

use crate::{
    Date, FromTimeScale, Gpst, LeapSecondError, LocalTime, Month, Tai, TaiTime, TimePoint,
    TimeScale, TryFromTimeScale, Unix, Utc,
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

    /// Terrestrial time is exactly (by definition) 32.184 seconds ahead of TAI. This means that
    /// its epoch is precisely 1977-01-01T00:00:00 TAI.
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        let date = Date::new(1977, Month::January, 1).unwrap();
        TaiTime::from_datetime(date, 0, 0, 0)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }

    /// For practical reasons (conversion to and from TCG), it is convenient to set the TT epoch to
    /// 1977-01-01T00:00:32.184: at this time, TT and TCG match exactly (by definition).
    fn epoch_local() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        let date = Date::new(1977, Month::January, 1).unwrap();
        let epoch = date.to_local_days().into_unit() + MilliSeconds::new(32_184);
        epoch.try_cast().unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl FromTimeScale<Tai> for Tt {}
impl FromTimeScale<Utc> for Tt {}
impl FromTimeScale<Gpst> for Tt {}

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
    let tai = TaiTime::from_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 32)
        .unwrap()
        .into_unit();
    let tt = TtTime::from_subsecond_datetime(
        Date::new(2004, Month::May, 14).unwrap(),
        16,
        44,
        4,
        MilliSeconds::new(184),
    )
    .unwrap();
    assert_eq!(tai, tt.into_time_scale());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::{Date, TaiTime};

    /// Verifies that construction of a terrestrial time from a historic date and time stamp never
    /// panics. An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TT epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > Date::new(i32::MIN / 8, Month::January, 1).unwrap());
        kani::assume(date < Date::new(i32::MAX / 8, Month::December, 31).unwrap());
        let _ = TtTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a terrestrial time from a Gregorian date and time stamp never
    /// panics. An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TT epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > GregorianDate::new(i32::MIN / 8, Month::January, 1).unwrap());
        kani::assume(date < GregorianDate::new(i32::MAX / 8, Month::December, 31).unwrap());
        let _ = TtTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid terrestrial time datetimes can be losslessly converted to and from
    /// the equivalent TAI time. An assumption is made on the input range because some dates result
    /// in a count of milliseconds from the TT epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn datetime_tai_roundtrip() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > Date::new(i32::MIN / 8, Month::January, 1).unwrap());
        kani::assume(date < Date::new(i32::MAX / 8, Month::December, 31).unwrap());
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1 = TtTime::from_datetime(date, hour, minute, second).unwrap();
        let tai: TaiTime<_, _> = time1.into_time_scale();
        let time2: TtTime<_, _> = tai.into_time_scale();
        assert_eq!(time1, time2);
    }
}
