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
    fn reference_epoch() -> TimePoint<Tai, i64, Milli> {
        Tai::reference_epoch().convert() - MilliSeconds::new(32_184)
    }

    /// Terrestrial time does not have an actual epoch associated with it. For practical purposes,
    /// it is useful to choose January 1, 1958, same as TAI.
    fn epoch<T>() -> LocalDays<T>
    where
        T: num::NumCast,
    {
        Tai::epoch()
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
        Ok(<() as TimeScaleConversion<Utc, Tt>>::convert(utc))
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
