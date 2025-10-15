//! Implementation of string parsing logic for `TimePoint` types.

use core::str::FromStr;

use crate::{
    Fraction, FromFineDateTime, HistoricDate, TimePoint, TryFromExact, TryMul, UnitRatio,
    errors::TimePointParsingError, parse::TimeOfDay, time_scale::TimeScale, units::Second,
};

impl<Scale, Representation, Period> FromStr for TimePoint<Scale, Representation, Period>
where
    Self: FromFineDateTime<Representation, Period>,
    Period: UnitRatio,
    Scale: TimeScale,
    Representation: TryFromExact<i64> + TryMul<Fraction, Output = Representation>,
{
    type Err = TimePointParsingError<<Self as FromFineDateTime<Representation, Period>>::Error>;

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

        let (time_of_day, mut string) = TimeOfDay::parse_partial(string)?;

        // Finally, the time point must end with a space, followed by the time zone abbreviation.
        if string.starts_with(" ") {
            string = string.get(1..).unwrap();
        } else {
            return Err(TimePointParsingError::ExpectedSpace);
        }

        if string.starts_with(Scale::ABBREVIATION) {
            string = string.get(Scale::ABBREVIATION.len()..).unwrap();
        } else {
            return Err(TimePointParsingError::ExpectedTimeScaleDesignator);
        }

        if !string.is_empty() {
            return Err(TimePointParsingError::UnexpectedRemainder);
        }

        let time_point = Self::from_fine_datetime(
            historic_date.into_date(),
            time_of_day.hour,
            time_of_day.minute,
            time_of_day.second,
            time_of_day
                .subseconds
                .convert_period::<Second, Period, Representation>()?,
        );

        match time_point {
            Ok(time_point) => Ok(time_point),
            Err(datetime_error) => Err(TimePointParsingError::DateTimeError(datetime_error)),
        }
    }
}

#[cfg(feature = "serde")]
impl<Scale, Representation, Period> serde::Serialize for TimePoint<Scale, Representation, Period>
where
    Self: ToString,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = self.to_string();
        serializer.serialize_str(&string)
    }
}

#[cfg(feature = "serde")]
impl<'de, Scale, Representation, Period> serde::Deserialize<'de>
    for TimePoint<Scale, Representation, Period>
where
    Self: FromStr,
    <Self as FromStr>::Err: core::fmt::Display,
    Scale: ?Sized,
    Period: ?Sized,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
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
    use crate::{Date, FromDateTime, IntoFineDateTime, TaiTime};
    let datetime = TaiTime::from_str(string).unwrap();
    let date = Date::from_historic_date(year, month, day).unwrap();
    let expected_datetime = TaiTime::<i64, Second>::from_datetime(date, hour, minute, second)
        .unwrap()
        .into_unit()
        + subseconds;
    assert_eq!(datetime, expected_datetime);
    let (date2, hour2, minute2, second2, subseconds2) = datetime.into_fine_datetime();
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
        "1958-01-01T00:00:00.000001 TAI",
        1958,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::new(1),
    );
    check_historic_datetime(
        "1958-01-02T00:00:00 TAI",
        1958,
        January,
        2,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1960-01-01T12:34:56.789123 TAI",
        1960,
        January,
        1,
        12,
        34,
        56,
        MicroSeconds::new(789123),
    );
    check_historic_datetime(
        "1961-01-01T00:00:00 TAI",
        1961,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1970-01-01T00:00:00 TAI",
        1970,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1976-01-01T23:59:59.999 TAI",
        1976,
        January,
        1,
        23,
        59,
        59,
        MicroSeconds::new(999000),
    );
    check_historic_datetime(
        "2025-07-16T16:23:24.000000000 TAI",
        2025,
        July,
        16,
        16,
        23,
        24,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "2034-12-26T08:02:37.123456000 TAI",
        2034,
        December,
        26,
        8,
        2,
        37,
        MicroSeconds::new(123456),
    );
    check_historic_datetime(
        "2760-04-01T21:59:58 TAI",
        2760,
        April,
        1,
        21,
        59,
        58,
        MicroSeconds::ZERO,
    );
    check_historic_datetime(
        "1643-01-04T01:01:33.000 TAI",
        1643,
        January,
        4,
        1,
        1,
        33,
        MicroSeconds::ZERO,
    );
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn check_historic_datetime_float(
    string: &str,
    year: i32,
    month: crate::Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    subseconds: crate::MicroSeconds<f64>,
) {
    use crate::{Date, FromDateTime, IntoFineDateTime, TaiTime};
    let datetime = TaiTime::from_str(string).unwrap();
    let date = Date::from_historic_date(year, month, day).unwrap();
    let expected_datetime = TaiTime::from_datetime(date, hour, minute, second)
        .unwrap()
        .try_cast()
        .unwrap()
        .into_unit()
        + subseconds;
    assert_eq!(datetime, expected_datetime);
    let (date2, hour2, minute2, second2, subseconds2) = datetime.into_fine_datetime();
    assert_eq!(date, date2);
    assert_eq!(hour, hour2);
    assert_eq!(minute, minute2);
    assert_eq!(second, second2);
    assert_eq!(subseconds, subseconds2);
}

#[test]
fn known_timestamps_float() {
    use crate::MicroSeconds;
    use crate::Month::*;
    use num_traits::ConstZero;

    check_historic_datetime_float(
        "1958-01-01T00:00:00.000001 TAI",
        1958,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::new(1.),
    );
    check_historic_datetime_float(
        "1958-01-02T00:00:00 TAI",
        1958,
        January,
        2,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime_float(
        "1960-01-01T12:34:56.789123 TAI",
        1960,
        January,
        1,
        12,
        34,
        56,
        MicroSeconds::new(789123.),
    );
    check_historic_datetime_float(
        "1961-01-01T00:00:00 TAI",
        1961,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime_float(
        "1970-01-01T00:00:00 TAI",
        1970,
        January,
        1,
        0,
        0,
        0,
        MicroSeconds::ZERO,
    );
    check_historic_datetime_float(
        "1976-01-01T23:59:59.999 TAI",
        1976,
        January,
        1,
        23,
        59,
        59,
        MicroSeconds::new(999000.),
    );
    check_historic_datetime_float(
        "2025-07-16T16:23:24.000000000 TAI",
        2025,
        July,
        16,
        16,
        23,
        24,
        MicroSeconds::ZERO,
    );
    check_historic_datetime_float(
        "2034-12-26T08:02:37.123456000 TAI",
        2034,
        December,
        26,
        8,
        2,
        37,
        MicroSeconds::new(123456.),
    );
    check_historic_datetime_float(
        "2760-04-01T21:59:58 TAI",
        2760,
        April,
        1,
        21,
        59,
        58,
        MicroSeconds::ZERO,
    );
    check_historic_datetime_float(
        "1643-01-04T01:01:33.000 TAI",
        1643,
        January,
        4,
        1,
        1,
        33,
        MicroSeconds::ZERO,
    );
}
