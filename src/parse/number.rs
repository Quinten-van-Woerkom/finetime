//! While parsing durations and time points, the parser may frequently encounter the scenario where
//! a parsed symbol may be a regular integer, but may also optionally contain a decimal fraction.
//! To more conveniently handle such scenarios, we write a generic implementation here.

use lexical_core::parse_partial;

use crate::errors::NumberParsingError;

/// Generic `Number` representation that may be used while parsing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Number {
    Integer(i64),
    Decimal {
        decimal: i64,
        fractional_digits: u32,
    },
}

impl Number {
    pub fn parse_partial(string: &str) -> Result<(Self, &str), NumberParsingError> {
        let (integer, parsed_bytes) = parse_partial::<i64>(string.as_bytes())?;
        let remainder = string.get(parsed_bytes..).unwrap();
        if let Some('.') = remainder.chars().next() {
            let remainder = remainder.get(1..).unwrap();
            let (decimal_fraction, fractional_digits) = parse_partial::<i64>(remainder.as_bytes())?;
            let remainder = remainder.get(fractional_digits..).unwrap();
            let fractional_digits = fractional_digits
                .try_into()
                .map_err(|_| NumberParsingError::TooManyFractionalDigits { fractional_digits })?;
            let decimal = integer * 10i64.pow(fractional_digits) + decimal_fraction;
            Ok((
                Number::Decimal {
                    decimal,
                    fractional_digits,
                },
                remainder,
            ))
        } else {
            Ok((Number::Integer(integer), remainder))
        }
    }
}
