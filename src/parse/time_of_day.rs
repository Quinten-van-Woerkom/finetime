//! Implementation of parsing for time-of-day. Also incorporates a helper struct that wraps such
//! combinations of hour, minute, and second.

use crate::{
    errors::{NumberParsingError, TimeOfDayParsingError},
    parse::DecimalNumber,
};

/// Wrapper for a time-of-day, as used primarily for parsing. Explicitly used only for parsing
/// because it cannot "correctly" encapsulate whether the given time is valid: an associated time
/// scale is needed for that, to determine whether leap seconds apply.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TimeOfDay {
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) subseconds: DecimalNumber,
}

impl TimeOfDay {
    /// Parses a time-of-day based on the input string. Accepts only the extended complete local
    /// time-of-day format described in ISO 8601, section 5.3.1.2. However, we do not accept a
    /// leading time designator ('T'). Rather, this designator is expected only in the full
    /// date-time parsing, to simplify the implementation (since this struct is not public-facing
    /// anyway).
    pub(crate) fn parse_partial(mut string: &str) -> Result<(Self, &str), TimeOfDayParsingError> {
        // Parse hour component
        let (hour, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        if consumed_bytes != 2 {
            return Err(TimeOfDayParsingError::HourRepresentationNotTwoDigits);
        }
        string = string.get(consumed_bytes..).unwrap();

        // Parse hour-minute delimiter
        if string.starts_with(':') {
            string = string.get(1..).unwrap();
        } else {
            return Err(TimeOfDayParsingError::ExpectedHourMinuteDelimiter);
        }

        // Parse minute component
        let (minute, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        if consumed_bytes != 2 {
            return Err(TimeOfDayParsingError::MinuteRepresentationNotTwoDigits);
        }
        string = string.get(consumed_bytes..).unwrap();

        // Parse minute-second delimiter
        if string.starts_with(':') {
            string = string.get(1..).unwrap();
        } else {
            return Err(TimeOfDayParsingError::ExpectedMinuteSecondDelimiter);
        }

        // Parse second component
        // First, we parse only the integer part
        let (second, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
        if consumed_bytes != 2 {
            return Err(TimeOfDayParsingError::IntegerSecondRepresentationNotTwoDigits);
        }
        string = string.get(consumed_bytes..).unwrap();

        // Then, we parse the fractional remainder, if any
        let subseconds = if string.starts_with('.') {
            string = string.get(1..).unwrap();
            let (fraction, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
            string = string.get(fractional_digits..).unwrap();
            let fractional_digits = fractional_digits
                .try_into()
                .map_err(|_| NumberParsingError::TooManyFractionalDigits { fractional_digits })?;
            DecimalNumber {
                integer: 0,
                fraction,
                fractional_digits,
            }
        } else {
            DecimalNumber::ZERO
        };

        Ok((
            TimeOfDay {
                hour,
                minute,
                second,
                subseconds,
            },
            string,
        ))
    }
}
