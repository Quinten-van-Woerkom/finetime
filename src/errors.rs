//! All errors that may be returned by public functions of this library are defined in this module.
//! This is useful in reducing the number of "unnecessary" inter-module dependencies, by ensuring
//! that using the results/error of a function does not require importing its entire module.

use thiserror::Error;

use crate::{DurationComponent, DurationDesignator, Month};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("{day} {month} {year} does not exist in the historic calendar")]
pub struct InvalidHistoricDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("{day} {month} {year} does not exist in the proleptic Gregorian calendar")]
pub struct InvalidGregorianDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("{day} {month} {year} does not exist in the proleptic Julian calendar")]
pub struct InvalidJulianDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid combination of year and day-of-year")]
pub enum InvalidDayOfYear {
    InvalidDayOfYearCount(#[from] InvalidDayOfYearCount),
    InvalidHistoricDate(#[from] InvalidHistoricDate),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("{day_of_year} is not a valid day in {year}")]
pub struct InvalidDayOfYearCount {
    pub day_of_year: u16,
    pub year: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid month number {month}")]
pub struct InvalidMonthNumber {
    pub month: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid week day number {week_day}")]
pub struct InvalidWeekDayNumber {
    pub week_day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid time-of-day {hour:02}-{minute:02}-{second:02}")]
pub struct InvalidTimeOfDay {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid historic date-time")]
pub enum InvalidHistoricDateTime<InvalidDateTime: core::error::Error> {
    InvalidHistoricDate(#[from] InvalidHistoricDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Gregorian date-time")]
pub enum InvalidGregorianDateTime<InvalidDateTime: core::error::Error> {
    InvalidGregorianDate(#[from] InvalidGregorianDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Julian date-time")]
pub enum InvalidJulianDateTime<InvalidDateTime: core::error::Error> {
    InvalidJulianDate(#[from] InvalidJulianDate),
    InvalidDateTime(#[source] InvalidDateTime),
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `Date`")]
pub enum HistoricDateParsingError {
    IntegerParsingError(#[from] lexical_core::Error),
    InvalidHistoricDate(#[from] InvalidHistoricDate),
    InvalidMonthNumber(#[from] InvalidMonthNumber),
    #[error("expected but did not find year-month delimiter '-'")]
    ExpectedYearMonthDelimiter,
    #[error("month representation must be exactly two digits")]
    MonthRepresentationNotTwoDigits,
    #[error("day representation must be exactly two digits")]
    DayRepresentationNotTwoDigits,
    #[error("expected but did not find month-day delimiter '-'")]
    ExpectedMonthDayDelimiter,
    #[error("could not parse entire string: data remains after historic date")]
    UnexpectedRemainder,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `Duration`")]
pub enum DurationParsingError {
    #[error("input string did not start with \'P\'")]
    ExpectedDurationPrefix,
    DurationDesignatorParsingError(#[from] DurationDesignatorParsingError),
    DurationComponentParsingError(#[from] DurationComponentParsingError),
    #[error(
        "unit designators must be provided in decreasing error, but found {current} after {previous}"
    )]
    NonDecreasingDesignators {
        previous: DurationDesignator,
        current: DurationDesignator,
    },
    #[error("only lowest order component may be expressed as decimal fraction")]
    OnlyLowestOrderComponentMayHaveDecimalFraction,
    DurationComponentConversionError(#[from] DurationComponentConversionError),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("unable to express duration component {component:?} in underlying representation")]
pub struct DurationComponentConversionError {
    pub component: DurationComponent,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("invalid duration component")]
pub enum DurationComponentParsingError {
    NumberParsingError(#[from] NumberParsingError),
    DurationDesignatorParsingError(#[from] DurationDesignatorParsingError),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid duration designator")]
pub enum DurationDesignatorParsingError {
    #[error("unexpected character: {character}")]
    UnexpectedCharacter { character: char },
    #[error("unexpected end-of-string")]
    UnexpectedEndOfString,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing number")]
pub enum NumberParsingError {
    ParsingError(#[from] lexical_core::Error),
    #[error("too many fractional digits: {fractional_digits}, only `i32::MAX` are supported")]
    TooManyFractionalDigits {
        fractional_digits: usize,
    },
}
