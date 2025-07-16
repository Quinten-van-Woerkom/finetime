//! Implementation of the Coordinated Universal Time (UTC) standard.

use num::Zero;
use tinyvec::ArrayVec;

use crate::{
    calendar::{
        Date, Datelike,
        Month::{self, *},
    },
    duration::units::{LiteralRatio, Milli},
    time_point::TimePoint,
    time_scale::{
        TimeScale, TimeScaleConversion,
        local::LocalDays,
        tai::{Tai, TaiTime},
        unix::{UnixTime, UnixTimeError},
    },
};

/// `UtcTime` is a specialization of `TimePoint` that uses the UTC time scale.
pub type UtcTime<Representation, Period = LiteralRatio<1>> = TimePoint<Utc, Representation, Period>;

impl UtcTime<i64> {
    /// Creates a UTC time point from a given calendar date and UTC time stamp inside of that day.
    /// Leap seconds are included, meaning that leap second days will have a 61st second in the
    /// last minute of their day.
    pub fn from_datetime(
        date: impl Datelike,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<UtcTime<i64>, UtcError> {
        Self::from_local_datetime(date.into(), hour, minute, second)
    }

    pub fn from_local_datetime(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<UtcTime<i64>, UtcError> {
        // A quick sanity check is used to catch "easy" failures early. We only check the seconds
        // count here because that is not caught by the steps executed next.
        if second > 60 || (second == 60 && hour != 23) {
            return Err(UtcError::InvalidDateTime {
                date,
                hour,
                minute,
                second,
            });
        }

        // Then, we compute the Unix time at this point in time. In that representation, leap
        // seconds are not incorporated, so we may compute it directly. Note that we do not compute
        // the seconds component, because that will require additional logic to handle leap
        // seconds.
        let unix_time_minutes = match UnixTime::from_datetime(date, hour, minute, 0) {
            Ok(unix_time) => unix_time,
            Err(UnixTimeError::TimeDoesNotExist {
                hour,
                minute,
                second,
            }) => {
                return Err(UtcError::InvalidDateTime {
                    date,
                    hour,
                    minute,
                    second,
                });
            }
        };
        // The seconds component is added afterwards, so that we create a full timestamp. We also
        // determine based on the timestamp whether a leap second is expected or not.
        let unix_time = unix_time_minutes + Seconds::new(second as i64);
        let expect_leap_second = second == 60;

        match LEAP_SECONDS.to_utc(unix_time) {
            // The nominal case: we do not expect a leap second, and we get a simple unambiguous
            // UTC time point back from the leap second table.
            LeapSecondsResult::Unambiguous(utc_time) if !expect_leap_second => Ok(utc_time),
            // If the second count is 60, we should have expected a leap second insertion to occur.
            // Hence, if we still find an unambiguous time stamp, that means that the requested
            // datetime does not actually exist, because there is no 61st second there.
            LeapSecondsResult::Unambiguous(_) => Err(UtcError::NoLeapSecondInsertion {
                date,
                hour,
                minute,
                second,
            }),
            // If the Unix time that we have created happens to be exactly at a leap second
            // insertion, we must manually disambiguate. We can do this by checking whether a
            // datetime was passed with a 61st (leap) second or not.
            LeapSecondsResult::InsertionPoint { start, end } => {
                if expect_leap_second {
                    Ok(start)
                } else {
                    Ok(end)
                }
            }
            // If the requested Unix time coincides with a leap second deletion, that means that we
            // cannot convert it to a valid UTC time.
            LeapSecondsResult::DeletionPoint => Err(UtcError::LeapSecondDeletion {
                date,
                hour,
                minute,
                second,
            }),
        }
    }
}

/// Representation of the Coordinated Universal Time (UTC) standard. UTC utilizes leap seconds to
/// synchronize the time to within 0.9 seconds of UT1, the solar time of the Earth. Since these
/// leap seconds may be updated, this struct makes use of a backing mutable static to permit
/// runtime changes to the leap second table, as needed for long-running programs.
///
/// The reason for choosing to do this over passing a pointer to this table every time is that it
/// would add a pointer of overhead to any UTC time stamp. Additionally, it would make UTC the only
/// stateful time scale, which would make generic code harder to work with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Utc;

impl Utc {
    /// Returns the UTC epoch as a date. Note that this date is itself still expressed in UTC,
    /// meaning that direct comparison to, for example, TAI is not possible.
    pub const fn epoch_as_date() -> Date {
        match Date::new(1970, Month::January, 1) {
            Ok(date) => date,
            Err(_) => panic!("Internal error: UTC epoch was found to be an invalid date."),
        }
    }
}

/// Tests the creation of UTC time points from calendar dates for some known values. We explicitly
/// try out times near leap second insertions to see if those are handled properly, including:
/// - Durations should be handled correctly before, during, and after a leap second.
/// - If a leap second format (61 seconds) datetime is given for a non-leap second datetime, this
///   shall be caught and indicated.
#[test]
fn calendar_dates_near_insertion() {
    // Leap second insertion of June 2015.
    let date = Date::new(2015, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::new(2015, July, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Leap second insertion of December 2016.
    let date = Date::new(2016, December, 31).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::new(2017, January, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Non-leap second date: June 2016
    let date = Date::new(2016, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60);
    assert_eq!(
        leap_second,
        Err(UtcError::NoLeapSecondInsertion {
            date: date.into(),
            hour: 23,
            minute: 59,
            second: 60
        })
    );
}

/// If a given conversion from Unix time to UTC succeeds or is ambiguous, all results shall map
/// back to the exact same Unix time.
#[test]
fn roundtrip_near_leap_seconds() {
    // Leap second insertion of June 2015.
    let date = Date::new(2015, June, 30).unwrap();
    let date2 = Date::new(2015, July, 1).unwrap();
    let date3 = Date::new(2016, December, 31).unwrap();
    let date4 = Date::new(2017, January, 1).unwrap();
    let date5 = Date::new(2016, June, 30).unwrap();

    let times = [
        UnixTime::from_datetime(date, 23, 59, 58).unwrap(),
        UnixTime::from_datetime(date, 23, 59, 59).unwrap(),
        UnixTime::from_datetime(date2, 0, 0, 0).unwrap(),
        UnixTime::from_datetime(date2, 0, 0, 1).unwrap(),
        UnixTime::from_datetime(date3, 23, 59, 58).unwrap(),
        UnixTime::from_datetime(date3, 23, 59, 59).unwrap(),
        UnixTime::from_datetime(date4, 0, 0, 0).unwrap(),
        UnixTime::from_datetime(date5, 23, 59, 58).unwrap(),
        UnixTime::from_datetime(date5, 23, 59, 59).unwrap(),
    ];

    for &time in times.iter() {
        match LEAP_SECONDS.to_utc(time) {
            LeapSecondsResult::Unambiguous(time_point) => {
                assert_eq!(LEAP_SECONDS.to_unix(time_point), time);
            }
            LeapSecondsResult::InsertionPoint { start, end } => {
                assert_eq!(LEAP_SECONDS.to_unix(start), time);
                assert_eq!(LEAP_SECONDS.to_unix(end), time);
            }
            LeapSecondsResult::DeletionPoint => {
                panic!("Unexpected deleted leap second found at {:?}", time)
            }
        }
    }
}

/// Errors that may be returned when creating a UTC time point from a calendar datetime. Note that
/// leap seconds may be included. Hence, this error will store the full timestamp when reporting
/// that the requested datetime does not exist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UtcError {
    /// Returned when the given combination of date and time-of-day is not a valid datetime in
    /// general (independent of the exact calendar).
    InvalidDateTime {
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    },
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
}

impl TimeScale for Utc {
    fn reference_epoch() -> TimePoint<Tai, i64, Milli> {
        let date = Date::new(1970, January, 1).unwrap();
        TaiTime::from_datetime(date, 0, 0, 10).unwrap().convert()
    }
}

impl TimeScaleConversion<Tai, Utc> for () {}
impl TimeScaleConversion<Utc, Tai> for () {}

/// Describes the evolution of leap seconds over time.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LeapSecondsTable {
    table: ArrayVec<[LeapSecondsEntry; 128]>,
}

impl LeapSecondsTable {
    /// Converts a given Unix timestamp to UTC by adding the relevant leap offset. Potentially
    /// ambiguous: right at the insertion point of a leap second, the same Unix time represents two
    /// UTC times (the leap second itself, and the time right after). Additionally, if a leap
    /// second deletion happens, there will be a Unix time stamp with no associated UTC time. This
    /// function handles all possibilities by returning an enumeration with the appropriate value.
    pub fn to_utc(&self, unix_time: UnixTime<i64>) -> LeapSecondsResult {
        let mut leap_second_offset = Seconds::new(0i64);
        for leap_second in self.table.iter() {
            // Since the table is sorted, if we find a Unix time later than our desired Unix time,
            // we may conclude that the previous leap second offset was the correct one.
            if leap_second.unix_time > unix_time {
                break;
            }
            // If we find a Unix time that coincides exactly with our desired Unix time, we must
            // inspect the leap second information to determine what is appropriate.
            if leap_second.unix_time == unix_time {
                match leap_second.event {
                    // If we find a leap second insertion, the Unix time stamp may be mapped to
                    // any time point within that range. We do not choose, but simply return the
                    // start (leap second, 23:59:60) and end (regular second, 00:00:00) of that
                    // range.
                    LeapSecondsEvent::Insertion => {
                        let time_since_epoch = unix_time.elapsed_time_since_epoch();
                        let start = time_since_epoch + leap_second_offset;
                        let start = UtcTime::from_time_since_epoch(start);
                        let end = time_since_epoch + leap_second.cumulative_offset;
                        let end = UtcTime::from_time_since_epoch(end);
                        return LeapSecondsResult::InsertionPoint { start, end };
                    }
                    // If we encounter a deletion, we return a flag indicating as such so that the
                    // caller can try to recover.
                    LeapSecondsEvent::Deletion => return LeapSecondsResult::DeletionPoint,
                }
            }
            // Otherwise, we update the leap second offset with the latest one and continue
            // searching for the next relevant one.
            leap_second_offset = leap_second.cumulative_offset;
        }
        let time_since_epoch = unix_time.elapsed_time_since_epoch() + leap_second_offset;
        let utc_time = UtcTime::from_time_since_epoch(time_since_epoch);
        LeapSecondsResult::Unambiguous(utc_time)
    }

    /// Converts a given UTC timestamp to Unix time by removing the leap seconds. Rather than the
    /// reverse transformation, this one will always succeed: all UTC times map to a Unix time,
    /// even if the reverse is not true.
    pub fn to_unix(&self, utc_time: UtcTime<i64>) -> UnixTime<i64> {
        for leap_second in self.table.iter().rev() {
            if leap_second.utc_time <= utc_time {
                let time_since_epoch = utc_time.elapsed_time_since_epoch();
                return UnixTime::from_time_since_epoch(
                    time_since_epoch - leap_second.cumulative_offset,
                );
            }
        }
        // If no leap seconds are found that were inserted before the UTC time, we can simply set
        // UTC equal to Unix time.
        UnixTime::from_time_since_epoch(utc_time.elapsed_time_since_epoch())
    }

    /// Adds a leap second update. Checks whether it is an insertion (likely) or deletion
    /// (unlikely) and computes the associated UTC time to speed up bidirectional lookups.
    pub fn insert(&mut self, seconds_since_unix_epoch: Seconds<i64>, offset: Seconds<i64>) {
        let unix_time = UnixTime::from_time_since_epoch(seconds_since_unix_epoch);
        let utc_time = UtcTime::from_time_since_epoch(seconds_since_unix_epoch + offset);
        let event = if let Some(entry) = self.table.last() {
            if entry.cumulative_offset < offset {
                LeapSecondsEvent::Insertion
            } else {
                LeapSecondsEvent::Deletion
            }
        } else {
            LeapSecondsEvent::Insertion
        };
        let no_room = self.table.try_push(LeapSecondsEntry {
            unix_time,
            utc_time,
            event,
            cumulative_offset: offset,
        });
        // If the table does not have enough room to push at this point, we just panic. At some
        // point in the future, it is probably better to have some backup that ensures continued
        // functioning, like merging leap second entries so that at least future offsets remain
        // valid.
        if no_room.is_some() {
            panic!("Ran out of room in leap second table");
        }
    }
}

/// Conversions from Unix time to UTC time are ambiguous, because insertions lead to folds in Unix
/// time (where the entire UTC leap second is mapped to a single Unix time) and deletions to gaps
/// (where a second of Unix time does not map to a real UTC time).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LeapSecondsResult {
    Unambiguous(UtcTime<i64>),
    InsertionPoint {
        start: UtcTime<i64>,
        end: UtcTime<i64>,
    },
    DeletionPoint,
}

/// Describes a leap second instance. Right after a leap second instance, the leap second offset
/// is set to the given value.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LeapSecondsEntry {
    unix_time: UnixTime<i64>,
    utc_time: UtcTime<i64>,
    event: LeapSecondsEvent,
    cumulative_offset: Seconds<i64>,
}

impl Default for LeapSecondsEntry {
    fn default() -> Self {
        Self {
            unix_time: UnixTime::from_time_since_epoch(Seconds::zero()),
            utc_time: UtcTime::from_time_since_epoch(Seconds::zero()),
            event: LeapSecondsEvent::Insertion,
            cumulative_offset: Seconds::zero(),
        }
    }
}

impl LeapSecondsEntry {
    /// Constructs a new leap second entry.
    pub fn new(
        seconds_since_unix_epoch: Seconds<i64>,
        total_leap_seconds: Seconds<i64>,
        event: LeapSecondsEvent,
    ) -> Self {
        Self {
            unix_time: UnixTime::from_time_since_epoch(seconds_since_unix_epoch),
            utc_time: UtcTime::from_time_since_epoch(seconds_since_unix_epoch + total_leap_seconds),
            event,
            cumulative_offset: total_leap_seconds,
        }
    }
}

/// Two kinds of leap second changes are possible:
/// - An insertion, as has historically always been the case. Here a 61st second is added to the
///   last minute of a day.
/// - A deletion, up to now theoretical. Here, the 60th second of the last minute of a day is
///   deleted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LeapSecondsEvent {
    Insertion,
    Deletion,
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// If a given conversion from Unix time to UTC succeeds or is ambiguous, all results shall map
    /// back to the exact same Unix time. A custom, constant leap second table is used for
    /// determinism.
    ///
    /// Note that this roundtrip characteristic only applies in the absence of leap second
    /// deletions. If a leap second is deleted, some Unix time will exist that does not map to an
    /// equivalent UTC time. Hence, that case must be checked separately, as there our
    /// implementation is not expected to provide full roundtrip capabilities.
    #[kani::proof]
    fn roundtrip_near_leap_seconds() {
        let mut leap_seconds = LeapSecondsTable::default();
        leap_seconds.insert(Seconds::new(63072000), Seconds::new(10));
        leap_seconds.insert(Seconds::new(78796800), Seconds::new(11));
        leap_seconds.insert(Seconds::new(94694400), Seconds::new(12));
        leap_seconds.insert(Seconds::new(126230400), Seconds::new(13));
        leap_seconds.insert(Seconds::new(157766400), Seconds::new(14));
        leap_seconds.insert(Seconds::new(189302400), Seconds::new(15));
        leap_seconds.insert(Seconds::new(220924800), Seconds::new(16));
        leap_seconds.insert(Seconds::new(252460800), Seconds::new(17));
        leap_seconds.insert(Seconds::new(283996800), Seconds::new(18));
        leap_seconds.insert(Seconds::new(315532800), Seconds::new(19));
        leap_seconds.insert(Seconds::new(362793600), Seconds::new(20));
        leap_seconds.insert(Seconds::new(394329600), Seconds::new(21));
        leap_seconds.insert(Seconds::new(425865600), Seconds::new(22));
        leap_seconds.insert(Seconds::new(489024000), Seconds::new(23));
        leap_seconds.insert(Seconds::new(567993600), Seconds::new(24));
        leap_seconds.insert(Seconds::new(631152000), Seconds::new(25));
        leap_seconds.insert(Seconds::new(662688000), Seconds::new(26));
        leap_seconds.insert(Seconds::new(709948800), Seconds::new(27));
        leap_seconds.insert(Seconds::new(741484800), Seconds::new(28));
        leap_seconds.insert(Seconds::new(773020800), Seconds::new(29));
        leap_seconds.insert(Seconds::new(820454400), Seconds::new(30));
        leap_seconds.insert(Seconds::new(867715200), Seconds::new(31));
        leap_seconds.insert(Seconds::new(915148800), Seconds::new(32));
        leap_seconds.insert(Seconds::new(1136073600), Seconds::new(33));
        leap_seconds.insert(Seconds::new(1230768000), Seconds::new(34));
        leap_seconds.insert(Seconds::new(1341100800), Seconds::new(35));
        leap_seconds.insert(Seconds::new(1435708800), Seconds::new(36));
        leap_seconds.insert(Seconds::new(1483228800), Seconds::new(37));

        let time: UnixTime<i64> = kani::any();
        kani::assume(time > UnixTime::from_time_since_epoch(Seconds::new(i64::MIN + 10)));
        kani::assume(time < UnixTime::from_time_since_epoch(Seconds::new(i64::MAX - 37)));

        match leap_seconds.to_utc(time) {
            LeapSecondsResult::Unambiguous(time_point) => {
                assert_eq!(leap_seconds.to_unix(time_point), time);
            }
            LeapSecondsResult::InsertionPoint { start, end } => {
                assert_eq!(leap_seconds.to_unix(start), time);
                assert_eq!(leap_seconds.to_unix(end), time);
            }
            LeapSecondsResult::DeletionPoint => {
                panic!("Unexpected deleted leap second found at {:?}", time)
            }
        }
    }

    /// If our leap second table has deletions, we cannot guarantee the roundtrip property around
    /// that time instance. However, we still want our implementation not to panic, and we can
    /// prove that.
    #[kani::proof]
    fn infallible_with_deletions() {
        let mut leap_seconds = LeapSecondsTable::default();
        leap_seconds.insert(Seconds::new(63072000), Seconds::new(0));
        leap_seconds.insert(Seconds::new(78796800), Seconds::new(1));
        leap_seconds.insert(Seconds::new(94694400), Seconds::new(2));
        leap_seconds.insert(Seconds::new(126230400), Seconds::new(3));
        leap_seconds.insert(Seconds::new(157766400), Seconds::new(4));
        leap_seconds.insert(Seconds::new(189302400), Seconds::new(5));
        leap_seconds.insert(Seconds::new(220924800), Seconds::new(6));
        leap_seconds.insert(Seconds::new(252460800), Seconds::new(7));
        leap_seconds.insert(Seconds::new(283996800), Seconds::new(8));
        leap_seconds.insert(Seconds::new(315532800), Seconds::new(9));
        leap_seconds.insert(Seconds::new(362793600), Seconds::new(10));
        leap_seconds.insert(Seconds::new(394329600), Seconds::new(11));
        leap_seconds.insert(Seconds::new(425865600), Seconds::new(12));
        leap_seconds.insert(Seconds::new(489024000), Seconds::new(13));
        leap_seconds.insert(Seconds::new(567993600), Seconds::new(14));
        leap_seconds.insert(Seconds::new(631152000), Seconds::new(15));
        leap_seconds.insert(Seconds::new(662688000), Seconds::new(16));
        leap_seconds.insert(Seconds::new(709948800), Seconds::new(17));
        leap_seconds.insert(Seconds::new(741484800), Seconds::new(18));
        leap_seconds.insert(Seconds::new(773020800), Seconds::new(19));
        leap_seconds.insert(Seconds::new(820454400), Seconds::new(20));
        leap_seconds.insert(Seconds::new(867715200), Seconds::new(21));
        leap_seconds.insert(Seconds::new(915148800), Seconds::new(22));
        leap_seconds.insert(Seconds::new(1136073600), Seconds::new(21));
        leap_seconds.insert(Seconds::new(1230768000), Seconds::new(22));
        leap_seconds.insert(Seconds::new(1341100800), Seconds::new(23));
        leap_seconds.insert(Seconds::new(1435708800), Seconds::new(24));
        leap_seconds.insert(Seconds::new(1483228800), Seconds::new(25));

        let time: UnixTime<i64> = kani::any();
        kani::assume(time > UnixTime::from_time_since_epoch(Seconds::new(i64::MIN)));
        kani::assume(time < UnixTime::from_time_since_epoch(Seconds::new(i64::MAX - 25)));

        let _ = leap_seconds.to_utc(time);
    }
}

// Load the leap seconds table as generated by the build script.
include!(concat!(env!("OUT_DIR"), "/leap_seconds.rs"));

#[test]
fn known_timestamps() {
    assert_eq!(
        UtcTime::from_datetime(Date::new(1970, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(0)
    );

    assert_eq!(
        UtcTime::from_datetime(Date::new(1973, Month::December, 31).unwrap(), 23, 59, 59)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230401)
    );

    assert_eq!(
        UtcTime::from_datetime(Date::new(1973, Month::December, 31).unwrap(), 23, 59, 60)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230402)
    );

    assert_eq!(
        UtcTime::from_datetime(Date::new(1974, Month::January, 1).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230403)
    );

    assert_eq!(
        UtcTime::from_datetime(Date::new(2025, Month::July, 16).unwrap(), 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752624027)
    );

    assert_eq!(
        UtcTime::from_datetime(Date::new(2025, Month::July, 16).unwrap(), 17, 36, 4)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752687391)
    );
}
