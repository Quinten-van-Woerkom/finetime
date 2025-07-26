//! Implementation of the Global Positioning System (GPS) time scale, generally abbreviated as
//! GPST.

use num::{NumCast, traits::NumOps};

use crate::{
    LocalDays, TimePoint, TryTimeScaleConversion, Unix, Utc,
    calendar::{Date, Month},
    time_scale::{
        TimeScale, TimeScaleConversion,
        tai::{Tai, TaiTime},
    },
    units::{LiteralRatio, Milli, Ratio},
};

/// `GpsTime` is a time point that is expressed according to the GPS time scale.
pub type GpsTime<Representation, Period = LiteralRatio<1>> =
    TimePoint<Gpst, Representation, Period>;

/// The Global Positioning System (GPS) time scale is broadcast by GPS satellites. It is based on
/// internal atomic clocks that are synchronized with TAI. The signal is defined to be a constant
/// 19 seconds behind TAI.
pub struct Gpst;

impl TimeScale for Gpst {
    fn reference_epoch() -> TimePoint<Tai, i64, Milli> {
        TaiTime::from_datetime(Date::new(1980, Month::January, 6).unwrap(), 0, 0, 19)
            .unwrap()
            .convert()
    }

    fn epoch<T>() -> LocalDays<T>
    where
        T: num::NumCast,
    {
        Date::new(1980, Month::January, 6)
            .unwrap()
            .to_local_days()
            .try_cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl TimeScaleConversion<Tai, Gpst> for () {}
impl TimeScaleConversion<Gpst, Tai> for () {}
impl TimeScaleConversion<Gpst, Utc> for () {}
impl TimeScaleConversion<Utc, Gpst> for () {}

impl<Representation, Period> TryTimeScaleConversion<Unix, Gpst, Representation, Period> for ()
where
    (): TryTimeScaleConversion<Unix, Utc, Representation, Period>,
    Period: Ratio,
    Representation: Copy + NumCast + NumOps,
{
    type Error = <() as TryTimeScaleConversion<Unix, Utc, Representation, Period>>::Error;

    fn try_convert(
        from: TimePoint<Unix, Representation, Period>,
    ) -> Result<TimePoint<Gpst, Representation, Period>, Self::Error> {
        let utc =
            <() as TryTimeScaleConversion<Unix, Utc, Representation, Period>>::try_convert(from)?;
        Ok(<() as TimeScaleConversion<Utc, Gpst>>::transform(utc))
    }
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    let tai = TaiTime::from_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 32).unwrap();
    let gpst =
        GpsTime::from_datetime(Date::new(2004, Month::May, 14).unwrap(), 16, 43, 13).unwrap();
    assert_eq!(tai, gpst.transform());
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a GPS time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GpsTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a GPS time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = GpsTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid GPS time datetimes can be losslessly converted to and from
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
        let time1 = GpsTime::from_datetime(date, hour, minute, second).unwrap();
        let tai: TaiTime<_> = time1.transform();
        let time2: GpsTime<_> = tai.transform();
        assert_eq!(time1, time2);
    }
}
