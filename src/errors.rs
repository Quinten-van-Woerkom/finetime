//! Description of all types of errors that may appear in this library. Separated into a separate
//! module to permit reuse without importing entire namespaces with unrelated functionality.

use crate::{Duration, LocalDays, Month, units::Unit};

/// Error that is returned if a date is encountered that does not exist in the historic calendar.
/// This may be either because the given day-of-month is not a valid day (for a given combination
/// of month and year) or because the given date falls within the ranges of dates skipped during
/// the Gregorian calendar reform (5 up to and including 14 October 1582).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DateDoesNotExist {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

/// Returned when a date is being created from a year and a day, but an invalid day-of-year is
/// passed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct YearDayDoesNotExist {
    pub year: i32,
    pub day_of_year: u16,
}

/// Error returned when the requested Gregorian date does not exist, because the requested
/// combination of month and year does not have the requested number of days.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GregorianDateDoesNotExist {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

/// Errors that may be returned when combining a calendar date with a time-of-day to create a
/// `TimePoint`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateTimeError {
    /// Returned when the given time-of-day does not exist in general (independent of whether the
    /// used time scale has leap seconds).
    InvalidTimeOfDay { hour: u8, minute: u8, second: u8 },
    /// Returned when the requested datetime has a 61st second but is not actually situated at a
    /// leap second insertion.
    NoLeapSecondInsertion {
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    },
    /// Returned when the requested datetime does not exist because of a leap second deletion.
    LeapSecondDeletion {
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    },
    /// Returned when the requested datetime could not fit in a `TimePoint` with the given
    /// `Representation`.
    NotRepresentable {
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    },
}

/// Errors that may be returned when combining a calendar date with a time-of-day and some given
/// number of subseconds to create a subsecond-accurate `TimePoint`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FineDateTimeError<T, P: Unit> {
    /// Wrapper for regular date time errors that are not specific to subsecond precision.
    DateTimeError(DateTimeError),
    /// The number of subseconds must be 0 or larger but also may not exceed 1 second.
    InvalidSubseconds { subseconds: Duration<T, P> },
}

impl<T, P> From<DateTimeError> for FineDateTimeError<T, P>
where
    P: Unit,
{
    fn from(value: DateTimeError) -> Self {
        Self::DateTimeError(value)
    }
}
