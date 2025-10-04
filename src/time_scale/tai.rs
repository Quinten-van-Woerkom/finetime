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
        Seconds::new(0)
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1958, Month::January, 2, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(86400),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1960, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(63072000),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1961, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(94694400),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1970, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(378691200),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1976, Month::January, 1, 0, 0, 0)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(567993600),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2025, Month::July, 16, 16, 23, 24)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(2131374204),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2034, Month::December, 26, 8, 2, 37)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(2429424157),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(2760, Month::April, 1, 21, 59, 58)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(25316575198),
    );

    assert_eq!(
        TaiTime::from_gregorian_datetime(1643, Month::January, 4, 1, 1, 33)
            .unwrap()
            .time_since_epoch(),
        Seconds::new(-9940143507),
    );
}

#[cfg(test)]
fn gregorian_datetime_roundtrip(
    year: i32,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) {
    use crate::GregorianDate;

    let time = TaiTime::from_gregorian_datetime(year, month, day, hour, minute, second).unwrap();
    let (date, hour2, minute2, second2) = time.to_datetime();
    let gregorian_date = GregorianDate::from_date(date);
    assert_eq!(gregorian_date.year(), year);
    assert_eq!(gregorian_date.month(), month);
    assert_eq!(gregorian_date.day(), day);
    assert_eq!(hour, hour2);
    assert_eq!(minute, minute2);
    assert_eq!(second, second2);
}

#[test]
fn date_decomposition() {
    gregorian_datetime_roundtrip(1999, Month::August, 22, 0, 0, 0);
    gregorian_datetime_roundtrip(1958, Month::January, 1, 0, 0, 0);
    gregorian_datetime_roundtrip(1958, Month::January, 2, 0, 0, 0);
    gregorian_datetime_roundtrip(1960, Month::January, 1, 0, 0, 0);
    gregorian_datetime_roundtrip(1961, Month::January, 1, 0, 0, 0);
    gregorian_datetime_roundtrip(1970, Month::January, 1, 0, 0, 0);
    gregorian_datetime_roundtrip(1976, Month::January, 1, 0, 0, 0);
    gregorian_datetime_roundtrip(2025, Month::July, 16, 16, 23, 24);
    gregorian_datetime_roundtrip(2034, Month::December, 26, 8, 2, 37);
    gregorian_datetime_roundtrip(2760, Month::April, 1, 21, 59, 58);
    gregorian_datetime_roundtrip(1643, Month::January, 4, 1, 1, 33);
    gregorian_datetime_roundtrip(1996, Month::January, 1, 3, 0, 0);
}
