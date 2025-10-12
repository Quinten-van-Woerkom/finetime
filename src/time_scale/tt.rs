//! Implementation of Terrestrial Time (TT).

use crate::{
    Date, Duration, MilliSeconds, Month, TimePoint,
    time_scale::{TerrestrialTime, TimeScale, datetime::ContinuousDateTimeScale},
    units::{Milli, Second},
};

pub type TtTime<Representation = i64, Period = Second> = TimePoint<Tt, Representation, Period>;

/// Time scale representing the Terrestrial Time (TT) scale. This scale is a constant 32.184
/// seconds ahead of TAI, but otherwise completely synchronized. It is used primarily as
/// independent variable in the context of planetary ephemerides.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tt;

impl TimeScale for Tt {
    const NAME: &'static str = "Terrestrial Time";

    const ABBREVIATION: &'static str = "TT";

    const EPOCH: Date<i32> = match Date::from_gregorian_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl ContinuousDateTimeScale for Tt {}

impl TerrestrialTime for Tt {
    type Representation = u16;
    type Period = Milli;
    const TAI_OFFSET: Duration<Self::Representation, Self::Period> = MilliSeconds::new(32_184);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, Month, TaiTime};
    let tai = TaiTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let tt = TtTime::from_fine_historic_datetime(
        2004,
        Month::May,
        14,
        16,
        44,
        4,
        crate::MilliSeconds::new(184i64),
    )
    .unwrap();
    assert_eq!(tai.into_unit(), tt.into_time_scale());
}

#[test]
fn date_decomposition() {
    let time =
        TtTime::<i64, Second>::from_historic_datetime(2004, Month::May, 14, 16, 44, 4).unwrap();
    let (date, hour, minute, second) = time.into_historic_datetime();
    assert_eq!(date.year(), 2004);
    assert_eq!(date.month(), Month::May);
    assert_eq!(date.day(), 14);
    assert_eq!(hour, 16);
    assert_eq!(minute, 44);
    assert_eq!(second, 4);
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a terrestrial time from a date and time stamp never
    /// panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        use crate::FromDateTime;
        let date: Date<i32> = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::<i64, Second>::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid terrestrial time datetimes can be losslessly converted to and from
    /// the equivalent TAI time.
    #[kani::proof]
    fn datetime_tai_roundtrip() {
        use crate::units::Milli;
        use crate::{FromDateTime, IntoTimeScale};
        let date: Date<i32> = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1 = TtTime::<i64, Milli>::from_datetime(date, hour, minute, second);
        if let Ok(time1) = time1 {
            let tai: TaiTime<i64, _> = time1.into_scale();
            let time2: TtTime<i64, _> = tai.into_scale();
            assert_eq!(time1, time2);
        }
    }
}
