//! Implementation of string parsing logic for `TimePoint` types.

use core::str::FromStr;

use crate::{
    ConvertUnit, DateTime, HistoricDate, TimePoint, UnitRatio, errors::TimePointParsingError,
    parse::TimeOfDay, units::Second,
};

impl<Scale, Period> FromStr for TimePoint<Scale, i64, Period>
where
    Scale: DateTime,
    Period: UnitRatio,
    i64: ConvertUnit<Second, Period>,
{
    type Err = TimePointParsingError<<Scale as DateTime>::Error>;

    /// Parses a `TimePoint` based on some ISO 8610 date and time of day string. Note that time
    /// shifts are explicitly not supported: those are already included in the choice of `Scale`
    /// for a type. Additionally, we only support the extended calendar date and time of day
    /// formats (see section 5.4.2.1 of ISO 8610). Finally, because the extended format is used
    /// (which explicitly delimits time point components), any number of digits is allowed in the
    /// year component, such that its range can be extended beyond the 0000..=9999 allowed by ISO
    /// 8601.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (historic_date, mut string) = HistoricDate::parse_partial(string)?;

        // Parse the mandatory time designator 'T'
        if string.starts_with("T") {
            string = string.get(1..).unwrap();
        } else {
            return Err(TimePointParsingError::ExpectedTimeDesignator);
        }

        let (time_of_day, string) = TimeOfDay::parse_partial(string)?;
        if !string.is_empty() {
            return Err(TimePointParsingError::UnexpectedRemainder);
        }

        let time_point = Scale::time_point_from_fine_datetime(
            historic_date.to_date(),
            time_of_day.hour,
            time_of_day.minute,
            time_of_day.second,
            time_of_day.subseconds.convert_period::<Second, Period>()?,
        );
        match time_point {
            Ok(time_point) => Ok(time_point),
            Err(datetime_error) => Err(TimePointParsingError::DateTimeError(datetime_error)),
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn check_historic_datetime(
    string: &str,
    year: i32,
    month: crate::Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    subseconds: crate::MicroSeconds<i64>,
) {
    use crate::{Date, Tai, TaiTime};
    let datetime = TaiTime::from_str(string).unwrap();
    let date = Date::from_historic_date(year, month, day).unwrap();
    let expected_datetime = TaiTime::<i64, _>::from_datetime(date, hour, minute, second)
        .unwrap()
        .into_unit()
        + subseconds;
    assert_eq!(datetime, expected_datetime);
    let (date2, hour2, minute2, second2, subseconds2) =
        Tai::fine_datetime_from_time_point(datetime);
    assert_eq!(date, date2);
    assert_eq!(hour, hour2);
    assert_eq!(minute, minute2);
    assert_eq!(second, second2);
    assert_eq!(subseconds, subseconds2);
}

#[test]
fn known_timestamps() {
    use crate::MicroSeconds;
    use crate::Month::*;
    use num_traits::ConstZero;

    check_historic_datetime(
        "1958-01-01T00:00:00.000001",
        1958,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::new(1),
    );
    check_historic_datetime(
        "1958-01-02T00:00:00",
        1958,
        January,
        2,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1960-01-01T12:34:56.789123",
        1960,
        January,
        1,
        12,
        34,
        56,
        MicroSeconds::new(789123),
    );
    check_historic_datetime(
        "1961-01-01T00:00:00",
        1961,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1970-01-01T00:00:00",
        1970,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1976-01-01T23:59:59.999",
        1976,
        January,
        1,
        23,
        59,
        59,
        MicroSeconds::new(999000),
    );
    check_historic_datetime(
        "2025-07-16T16:23:24.000000000",
        2025,
        July,
        16,
        16,
        23,
        24,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "2034-12-26T08:02:37.123456000",
        2034,
        December,
        26,
        8,
        2,
        37,
        MicroSeconds::new(123456),
    );
    check_historic_datetime(
        "2760-04-01T21:59:58",
        2760,
        April,
        1,
        21,
        59,
        58,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1643-01-04T01:01:33.000",
        1643,
        January,
        4,
        1,
        1,
        33,
        MicroSeconds::ZERO,
    );
}
