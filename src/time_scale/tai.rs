//! Implementation of international atomic time (TAI).

use num::Zero;

use crate::{
    FromTimeScale, Gpst, LeapSecondError, LocalTime, Seconds, TryFromTimeScale, Tt, Unix, Utc,
    arithmetic::{FromUnit, Second, TimeRepresentation, Unit},
    calendar::{Date, Month},
    time_point::TimePoint,
    time_scale::TimeScale,
};

/// `TaiTime` is a specialization of `TimePoint` that uses the TAI time scale.
pub type TaiTime<Representation, Period = Second> = TimePoint<Tai, Representation, Period>;

/// Time scale representing the international atomic time standard (TAI). TAI has no leap seconds
/// and increases monotonically at a constant interval, making it useful as fundamental time scale
/// to build the rest of this library on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tai;

impl TimeScale for Tai {
    type NativePeriod = Second;

    /// Since TAI is used as central time scale, its own reference epoch is at time point 0.
    fn epoch_tai<T>() -> TaiTime<T, Self::NativePeriod>
    where
        T: TimeRepresentation,
    {
        TimePoint::from_time_since_epoch(Seconds::<i64>::zero().try_cast().unwrap())
    }

    fn epoch_local<T>() -> LocalTime<T, Self::NativePeriod>
    where
        T: TimeRepresentation,
    {
        Date::new(1958, Month::January, 1)
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

impl FromTimeScale<Tt> for Tai {}
impl FromTimeScale<Gpst> for Tai {}
impl FromTimeScale<Utc> for Tai {}

impl TryFromTimeScale<Unix> for Tai {
    type Error = LeapSecondError;

    fn try_from_time_scale<Representation, Period>(
        from: TimePoint<Unix, Representation, Period>,
    ) -> Result<TimePoint<Self, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<<Unix as TimeScale>::NativePeriod, Representation>
            + FromUnit<Self::NativePeriod, Representation>,
        Representation: TimeRepresentation,
    {
        let utc_time = Utc::try_from_time_scale(from)?;
        Ok(utc_time.into_time_scale())
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TAI time from a historic date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TaiTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a TAI time from a Gregorian date and time stamp never panics.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TaiTime::from_datetime(date, hour, minute, second);
    }
}

/// Verifies this implementation by computing the `TaiTime` for some known time stamps. To compute
/// these known time stamps, we use the fact that TAI, just like Unix time, has days that are
/// always exactly 86,400 seconds long. Hence, the differences are caused by only an offset in
/// epoch, which is the difference between 1958 and 1970: 378691200 seconds.
#[test]
fn known_timestamps() {
    use crate::duration::Seconds;
    assert_eq!(
        TaiTime::from_datetime(Date::new(1958, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(0)
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1958, Month::January, 2).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1960, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2 * 365 * 24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1961, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new((3 * 365 + 1) * 24 * 60 * 60),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1970, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1976, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(189302400 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2025, Month::July, 16).unwrap(), 16, 23, 24)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752683004 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2034, Month::December, 26).unwrap(), 8, 2, 37)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(2050732957 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(2760, Month::April, 1).unwrap(), 21, 59, 58)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(24937883998 + 378691200),
    );

    assert_eq!(
        TaiTime::from_datetime(Date::new(1643, Month::January, 4).unwrap(), 1, 1, 33)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(-10318834707 + 378691200),
    );
}
