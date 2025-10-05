//! While parsing durations and time points, the parser may frequently encounter the scenario where
//! a parsed symbol may be a regular integer, but may also optionally contain a decimal fraction.
//! To more conveniently handle such scenarios, we write a generic implementation here.

use lexical_core::parse_partial;

use crate::{
    Duration, Fraction, TryMul, UnitRatio,
    errors::{CannotRepresentDecimalNumber, NumberParsingError},
};

/// Generic `Number` representation that may be used while parsing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DecimalNumber {
    pub(crate) integer: i64,
    pub(crate) fraction: i64,
    pub(crate) fractional_digits: u32,
}

impl DecimalNumber {
    /// Parses a decimal number. Does not need to consume the entire input string.
    pub(crate) fn parse_partial(string: &str) -> Result<(Self, &str), NumberParsingError> {
        let (integer, parsed_bytes) = parse_partial::<i64>(string.as_bytes())?;
        let remainder = string.get(parsed_bytes..).unwrap();
        if let Some('.') = remainder.chars().next() {
            let remainder = remainder.get(1..).unwrap();
            let (fraction, fractional_digits) = parse_partial::<i64>(remainder.as_bytes())?;
            let remainder = remainder.get(fractional_digits..).unwrap();
            let fractional_digits = fractional_digits
                .try_into()
                .map_err(|_| NumberParsingError::TooManyFractionalDigits { fractional_digits })?;
            if fraction == 0 {
                Ok((
                    DecimalNumber {
                        integer,
                        fraction: 0,
                        fractional_digits: 0,
                    },
                    remainder,
                ))
            } else {
                Ok((
                    DecimalNumber {
                        integer,
                        fraction,
                        fractional_digits,
                    },
                    remainder,
                ))
            }
        } else {
            Ok((
                DecimalNumber {
                    integer,
                    fraction: 0,
                    fractional_digits: 0,
                },
                remainder,
            ))
        }
    }

    /// Returns whether this decimal number is fully integer: i.e., whether its fractional part is
    /// zero.
    pub(crate) fn is_integer(&self) -> bool {
        self.fractional_digits == 0
    }

    /// While parsing, it is common to encounter scenarios like "3.142 s" where a decimal number,
    /// expressed in one unit (seconds, here) but must be converted into the actual "underlying"
    /// representation of higher accuracy (milliseconds, here).
    ///
    /// This function provides the ability to do such a conversion: given the parsed decimal number
    /// "3.142", it losslessly converts it from the expressed unit `From` into the target unit
    /// `Into`. If this conversion cannot be done exactly, raises an appropriate error.
    pub(crate) fn convert_period<From, Into>(
        self,
    ) -> Result<Duration<i64, Into>, CannotRepresentDecimalNumber>
    where
        From: UnitRatio,
        Into: UnitRatio,
    {
        let fraction = Fraction::new(1, 10u64.pow(self.fractional_digits));
        let mantissa = if self.integer >= 0 {
            10i64.pow(self.fractional_digits) * self.integer + self.fraction
        } else {
            10i64.pow(self.fractional_digits) * self.integer - self.fraction
        };
        let uncorrected_duration = Duration::<_, From>::new(mantissa)
            .try_into_unit()
            .ok_or(CannotRepresentDecimalNumber { number: self })?;
        uncorrected_duration
            .try_mul(fraction)
            .ok_or(CannotRepresentDecimalNumber { number: self })
    }

    /// Decimal number that evaluates to zero.
    pub(crate) const ZERO: Self = Self {
        integer: 0,
        fraction: 0,
        fractional_digits: 0,
    };
}
