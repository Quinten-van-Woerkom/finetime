//! Implementation of string parsing logic for `JulianDate` types.

use core::str::FromStr;

use crate::{JulianDate, Month, errors::JulianDateParsingError};

impl FromStr for JulianDate {
    type Err = JulianDateParsingError;

    /// Parses a `JulianDate` based on some string. Accepts only the extended complete calendar
    /// date format specified in ISO 8601 (see section 5.2.2.1), though in addition any number of
    /// digits is accepted for each term.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (date, remainder) = Self::parse_partial(string)?;
        if !remainder.is_empty() {
            Err(JulianDateParsingError::UnexpectedRemainder)
        } else {
            Ok(date)
        }
    }
}

impl JulianDate {
    /// Parses a `JulianDate` based on some string. Accepts only the extended complete calendar
    /// date format specified in ISO 8601 (see section 5.2.2.1), though in addition any number of
    /// digits is accepted for the years term - to extend applicability of the format to a larger
    /// time range.
    ///
    /// On success, returns the resulting `JulianDate` and any remaining input that was not yet
    /// parsed. On failure, returns a reason indicating why.
    pub fn parse_partial(mut string: &str) -> Result<(Self, &str), JulianDateParsingError> {
        // Parse year component
        let (year, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        string = string.get(consumed_bytes..).unwrap();

        // Parse year-month delimiter
        if string.starts_with('-') {
            string = string.get(1..).unwrap();
        } else {
            return Err(JulianDateParsingError::ExpectedYearMonthDelimiter);
        }

        let (month, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        if consumed_bytes != 2 {
            return Err(JulianDateParsingError::MonthRepresentationNotTwoDigits);
        }
        let month = Month::try_from(month)?;
        string = string.get(consumed_bytes..).unwrap();

        // Parse month-day delimiter
        if string.starts_with('-') {
            string = string.get(1..).unwrap();
        } else {
            return Err(JulianDateParsingError::ExpectedMonthDayDelimiter);
        }

        // Parse day component
        let (day, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        if consumed_bytes != 2 {
            return Err(JulianDateParsingError::DayRepresentationNotTwoDigits);
        }
        string = string.get(consumed_bytes..).unwrap();

        Ok((JulianDate::new(year, month, day)?, string))
    }
}

/// Tests whether a given string parses to the same Julian date as the passed year, month, and
/// day arguments.
#[cfg(test)]
fn parse_known_julian_date(input: &str, year: i32, month: Month, day: u8) {
    let date = JulianDate::from_str(input).unwrap();
    let expected_date = JulianDate::new(year, month, day).unwrap();
    assert_eq!(date, expected_date);
}

/// Verifies string parsing for some known valid dates.
#[test]
fn known_dates() {
    use crate::Month::*;
    parse_known_julian_date("2000-01-01", 2000, January, 1);
    parse_known_julian_date("1999-01-01", 1999, January, 1);
    parse_known_julian_date("1987-06-19", 1987, June, 19);
    parse_known_julian_date("1988-01-27", 1988, January, 27);
    parse_known_julian_date("1988-06-19", 1988, June, 19);
    parse_known_julian_date("1900-01-01", 1900, January, 1);
    parse_known_julian_date("1600-01-01", 1600, January, 1);
    parse_known_julian_date("1600-12-31", 1600, December, 31);
    parse_known_julian_date("837-04-10", 837, April, 10);
    parse_known_julian_date("-123-12-31", -123, December, 31);
    parse_known_julian_date("-122-01-01", -122, January, 1);
    parse_known_julian_date("-1000-07-12", -1000, July, 12);
    parse_known_julian_date("-1000-02-29", -1000, February, 29);
    parse_known_julian_date("-1001-08-17", -1001, August, 17);
    parse_known_julian_date("-4712-01-01", -4712, January, 1);
}
