//! Implementation of International Atomic Time (TAI).

use crate::{Date, Month, TimePoint, time_scale::datetime::ContinuousDateTime, units::Second};

pub type TaiTime<Representation, Period = Second> = TimePoint<Tai, Representation, Period>;

/// Time scale representing the International Atomic Time standard (TAI). TAI has no leap seconds
/// and increases monotonically at a constant rate. This makes it highly suitable for scientific
/// and high-accuracy timekeeping.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tai;

impl ContinuousDateTime for Tai {
    const EPOCH: Date<i32> = match Date::from_gregorian_date(1958, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

/// Verifies this implementation by computing the `TaiTime` for some known (computed manually or
/// obtained elsewhere) time stamps.
#[test]
fn known_timestamps() {
    use crate::duration::Seconds;

    assert_eq!(
        TaiTime::from_gregorian_datetime(1958, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(0i32)
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1958, Month::January, 2, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(86400i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1960, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(63072000i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1961, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(94694400i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1970, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(378691200i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1976, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(567993600i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2025, Month::July, 16, 16, 23, 24)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(2131374204i32),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2034, Month::December, 26, 8, 2, 37)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(2429424157i64),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2760, Month::April, 1, 21, 59, 58)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(25316575198i64),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1643, Month::January, 4, 1, 1, 33)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(-9940143507i64),
    );
}
