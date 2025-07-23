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
        Ok(<() as TimeScaleConversion<Utc, Gpst>>::convert(utc))
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
