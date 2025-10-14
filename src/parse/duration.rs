//! Implementation of parsing logic for `Duration` types.

use core::str::FromStr;

use num_traits::ConstZero;

use crate::{
    Duration, UnitRatio,
    errors::{
        CannotRepresentDecimalNumber, DurationComponentParsingError,
        DurationDesignatorParsingError, DurationParsingError,
    },
    parse::DecimalNumber,
    units::{Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute, SecondsPerYear},
};

impl<Period> FromStr for Duration<i64, Period>
where
    Period: UnitRatio,
{
    type Err = DurationParsingError;

    /// Parses a `Duration` type based on some ISO 8601 duration string. However, we additionally
    /// impose that months may not be used as duration, to prevent confusion with minutes (and
    /// because their precise duration cannot be unambiguously defined). Furthermore, we do not
    /// support use of the time designator ('T') inside duration expressions. Finally, we support
    /// years, days, hours, minutes, and seconds with any number of digits.
    ///
    /// For years, following the rest of `finetime`, a duration of 31556952 seconds is used, which
    /// corresponds with the exact average duration of a Gregorian year.
    fn from_str(mut string: &str) -> Result<Self, Self::Err> {
        // Parse the mandatory duration prefix 'P'.
        if string.starts_with("P") {
            string = string.get(1..).unwrap();
        } else {
            return Err(DurationParsingError::ExpectedDurationPrefix);
        }

        let mut duration = Self::ZERO;
        let mut previous_designator = None;

        loop {
            let (component, remainder) = DurationComponent::parse_partial(string)?;
            string = remainder;

            // Verify that the units are provided in decreasing order.
            if let Some(previous) = previous_designator {
                if component.designator >= previous {
                    return Err(DurationParsingError::NonDecreasingDesignators {
                        current: component.designator,
                        previous,
                    });
                }
                previous_designator = Some(component.designator);
            }

            duration += component.into_period()?;

            if component.has_decimal_fraction() && !string.is_empty() {
                return Err(DurationParsingError::OnlyLowestOrderComponentMayHaveDecimalFraction);
            }

            if string.is_empty() {
                return Ok(duration);
            }
        }
    }
}

/// A duration is represented as multiple "duration components", each consisting of a number and a
/// duration designator.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DurationComponent {
    number: DecimalNumber,
    designator: DurationDesignator,
}

impl DurationComponent {
    /// Tries to parse a duration component from a string.
    pub fn parse_partial(string: &str) -> Result<(Self, &str), DurationComponentParsingError> {
        let (number, remainder) = DecimalNumber::parse_partial(string)?;
        let (designator, remainder) = DurationDesignator::parse_partial(remainder)?;
        Ok((Self { number, designator }, remainder))
    }

    /// If a duration component contains a decimal fraction instead of a integer, it must per ISO
    /// 8601 be the last duration component in a duration string.
    fn has_decimal_fraction(&self) -> bool {
        !self.number.is_integer()
    }

    /// Tries to convert a parsed duration component into the equivalent underlying representation
    /// for some given unit.
    fn into_period<Period>(self) -> Result<Duration<i64, Period>, CannotRepresentDecimalNumber>
    where
        Period: UnitRatio,
    {
        match self.designator {
            DurationDesignator::Seconds => self.number.convert_period::<Second, Period, _>(),
            DurationDesignator::Minutes => {
                self.number.convert_period::<SecondsPerMinute, Period, _>()
            }
            DurationDesignator::Hours => self.number.convert_period::<SecondsPerHour, Period, _>(),
            DurationDesignator::Days => self.number.convert_period::<SecondsPerDay, Period, _>(),
            DurationDesignator::Years => self.number.convert_period::<SecondsPerYear, Period, _>(),
        }
    }
}

/// The set of duration symbols that are supported when expressing durations as strings.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
pub enum DurationDesignator {
    Seconds,
    Minutes,
    Hours,
    Days,
    Years,
}

impl DurationDesignator {
    /// Tries to parse a (part of) a string into a duration designator. On success, returns the
    /// parsed symbol as well as the remainder of the string that was not parsed. On error, returns
    /// an `InvalidDurationDesignator` indicating the reason for failure.
    pub fn parse_partial(string: &str) -> Result<(Self, &str), DurationDesignatorParsingError> {
        match string.chars().next() {
            None => Err(DurationDesignatorParsingError::UnexpectedEndOfString),
            Some(character) => {
                let string = string.get(1..).unwrap();
                let symbol = match character {
                    'Y' => DurationDesignator::Years,
                    'D' => DurationDesignator::Days,
                    'H' => DurationDesignator::Hours,
                    'M' => DurationDesignator::Minutes,
                    'S' => DurationDesignator::Seconds,
                    _ => {
                        return Err(DurationDesignatorParsingError::UnexpectedCharacter {
                            character,
                        });
                    }
                };
                Ok((symbol, string))
            }
        }
    }
}

/// Tests that "simple" durations, made up of only one unit, can correctly be constructed.
#[test]
fn simple_durations() {
    use crate::{Days, Hours, Minutes, Seconds, Years};

    let second = Seconds::from_str("P1S").unwrap();
    assert_eq!(second, Seconds::new(1));
    let seconds = Seconds::from_str("P42S").unwrap();
    assert_eq!(seconds, Seconds::new(42));

    let minute = Minutes::from_str("P1M").unwrap();
    assert_eq!(minute, Minutes::new(1));
    let minutes = Minutes::from_str("P1998M").unwrap();
    assert_eq!(minutes, Minutes::new(1998));

    let hour = Hours::from_str("P1H").unwrap();
    assert_eq!(hour, Hours::new(1));
    let hours = Hours::from_str("P76H").unwrap();
    assert_eq!(hours, Hours::new(76));

    let day = Days::from_str("P1D").unwrap();
    assert_eq!(day, Days::new(1));
    let days = Days::from_str("P31415D").unwrap();
    assert_eq!(days, Days::new(31415));

    let year = Years::from_str("P1Y").unwrap();
    assert_eq!(year, Years::new(1));
    let years = Years::from_str("P2000Y").unwrap();
    assert_eq!(years, Years::new(2000));
}

/// Verifies that composite durations can be constructed.
#[test]
fn composite_durations() {
    use crate::Seconds;
    let duration = Seconds::from_str("P1Y2D3H4M5S").unwrap();
    assert_eq!(
        duration,
        Seconds::new(31556952 + 2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );
}

/// Verifies that it is possible to construct durations from sub-unit duration components as long
/// as the components can exactly be converted into the representation unit (e.g., 60 minutes can
/// be converted into an hour, so "P60M" is a valid representation for hours).
#[test]
fn sub_unit_durations() {
    use crate::Hours;
    let hour = Hours::from_str("P60M").unwrap();
    assert_eq!(hour, Hours::new(1));
}

/// Checks whether fractional duration representations can be constructed.
#[test]
fn fractional_durations() {
    use crate::{MilliSeconds, Seconds};
    let milliseconds = MilliSeconds::from_str("P5.123S").unwrap();
    assert_eq!(milliseconds, MilliSeconds::new(5123));

    let milliseconds = MilliSeconds::from_str("P23H59M58.123S").unwrap();
    assert_eq!(
        milliseconds,
        MilliSeconds::new(58123 + 59 * 60_000 + 23 * 3_600_000)
    );

    let seconds = Seconds::from_str("P23H59.5M").unwrap();
    assert_eq!(seconds, Seconds::new(23 * 3600 + 59 * 60 + 30));
}
