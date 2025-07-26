//! Implementation of the Terrestrial Time (TT) time scale.

use num::{NumCast, traits::NumOps};

use crate::{
    LocalDays, TimePoint, TryTimeScaleConversion, Unix, Utc,
    duration::MilliSeconds,
    time_scale::{Tai, TimeScale, TimeScaleConversion},
    units::{LiteralRatio, Milli, Ratio},
};

/// A time point that is expressed in Terrestrial Time.
pub type TtTime<Representation, Period = LiteralRatio<1>> = TimePoint<Tt, Representation, Period>;

/// Terrestrial time is the proper time of a clock located on the Earth geoid. It is used in
/// astronomical tables, mostly. Effectively, it is little more than a constant offset from TAI.
pub struct Tt;

impl TimeScale for Tt {
    /// Terrestrial time is exactly (by definition) 32.184 seconds ahead of TAI.
    fn epoch_tai() -> TimePoint<Tai, i64, Milli> {
        Tai::epoch_tai().convert() - MilliSeconds::new(32_184)
    }

    /// Terrestrial time does not have an actual epoch associated with it. For practical purposes,
    /// it is useful to choose January 1, 1958, same as TAI.
    fn epoch_local<T>() -> LocalDays<T>
    where
        T: num::NumCast,
    {
        Tai::epoch_local()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl TimeScaleConversion<Tt, Tai> for () {}
impl TimeScaleConversion<Tai, Tt> for () {}
impl TimeScaleConversion<Tt, Utc> for () {}
impl TimeScaleConversion<Utc, Tt> for () {}

impl<Representation, Period> TryTimeScaleConversion<Unix, Tt, Representation, Period> for ()
where
    (): TryTimeScaleConversion<Unix, Utc, Representation, Period>,
    Period: Ratio,
    Representation: Copy + NumCast + NumOps,
{
    type Error = <() as TryTimeScaleConversion<Unix, Utc, Representation, Period>>::Error;

    fn try_convert(
        from: TimePoint<Unix, Representation, Period>,
    ) -> Result<TimePoint<Tt, Representation, Period>, Self::Error> {
        let utc =
            <() as TryTimeScaleConversion<Unix, Utc, Representation, Period>>::try_convert(from)?;
        Ok(<() as TimeScaleConversion<Utc, Tt>>::transform(utc))
    }
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    use crate::{Date, Month, TaiTime};
    let tai = TaiTime::from_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 32)
        .unwrap()
        .convert();
    let tt = TtTime::from_subsecond_datetime(
        Date::new(2004, Month::May, 14).unwrap(),
        16,
        44,
        4,
        MilliSeconds::new(184),
    )
    .unwrap();
    assert_eq!(tai, tt.transform());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::{Date, TaiTime};

    /// Verifies that construction of a terrestrial time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a terrestrial time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid terrestrial time datetimes can be losslessly converted to and from
    /// the equivalent TAI time.
    #[kani::proof]
    fn datetime_tai_roundtrip() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1 = TtTime::from_datetime(date, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.transform();
        let time2: TtTime<_> = tai.transform();
        assert_eq!(time1, time2);
    }
}
