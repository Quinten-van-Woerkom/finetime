//! Implementation of the Coordinated Universal Time (UTC) standard.

use core::fmt::Debug;

use num_traits::Zero;
use tinyvec::ArrayVec;

use crate::{
    DateTimeError, FromTimeScale, LocalDays, LocalTime, TaiTime, TerrestrialTimeScale, TimeScale,
    TryFromTimeScale, Unix, UnixTime,
    arithmetic::{
        FromUnit, IntoUnit, Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute,
        TimeRepresentation, TryFromExact, Unit,
    },
    calendar::{
        Date,
        Month::{self, *},
    },
    duration::Duration,
    time_point::TimePoint,
};

/// `UtcTime` is a specialization of `TimePoint` that uses the UTC time scale.
pub type UtcTime<Representation, Period = Second> = TimePoint<Utc, Representation, Period>;

/// Representation of the Coordinated Universal Time (UTC) standard. UTC utilizes leap seconds to
/// synchronize the time to within 0.9 seconds of UT1, the solar time of the Earth. Since these
/// leap seconds may be updated, this struct makes use of a backing mutable static to permit
/// runtime changes to the leap second table, as needed for long-running programs.
///
/// The reason for choosing to do this over passing a pointer to this table every time is that it
/// would add a pointer of overhead to any UTC time stamp. Additionally, it would make UTC the only
/// stateful time scale, which would make generic code harder to work with.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl TimeScale for Utc {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    /// Because the UTC epoch coincides with the `LocalDays` epoch, it can be constructed simply
    /// as a zero value.
    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        LocalDays::from_time_since_epoch(Duration::new(0i64))
            .into_unit()
            .try_cast()
            .unwrap()
    }

    fn from_local_datetime(
        date: LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, i64>, DateTimeError>
    where
        SecondsPerDay: IntoUnit<Second, i64>,
        SecondsPerHour: IntoUnit<Second, i64>,
        SecondsPerMinute: IntoUnit<Second, i64>,
        Second: IntoUnit<Second, i64>,
    {
        // Verify that the time-of-day is valid. Leap seconds are always assumed to be valid at
        // this point - this is checked later.
        if hour >= 24
            || minute >= 60
            || second > 60
            || (second == 60 && (hour != 23 || minute != 59))
        {
            return Err(DateTimeError::InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }

        // Then, we compute the Unix time at this point in time. In that representation, leap
        // seconds are not incorporated, so we may compute it directly. Note that we do not compute
        // the seconds component, because that will require additional logic to handle leap
        // seconds.
        let unix_time_minutes = match UnixTime::from_generic_datetime(date, hour, minute, 0) {
            Ok(unix_time) => unix_time,
            _ => unreachable!(),
        };
        // The seconds component is added afterwards, so that we create a full timestamp. We also
        // determine based on the timestamp whether a leap second is expected or not.
        let unix_time = unix_time_minutes + Seconds::new(second).cast();
        let unix_time = unix_time
            .try_cast()
            .ok_or(DateTimeError::NotRepresentable {
                date,
                hour,
                minute,
                second,
            })?;
        let expect_leap_second = second == 60;

        match LEAP_SECONDS.to_utc(unix_time) {
            // The nominal case: we do not expect a leap second, and we get a simple unambiguous
            // UTC time point back from the leap second table.
            LeapSecondsResult::Unambiguous(utc_time) if !expect_leap_second => {
                utc_time.try_cast().ok_or(DateTimeError::NotRepresentable {
                    date,
                    hour,
                    minute,
                    second,
                })
            }
            // If the second count is 60, we should have expected a leap second insertion to occur.
            // Hence, if we still find an unambiguous time stamp, that means that the requested
            // datetime does not actually exist, because there is no 61st second there.
            LeapSecondsResult::Unambiguous(_) => Err(DateTimeError::NoLeapSecondInsertion {
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
                    start.try_cast().ok_or(DateTimeError::NotRepresentable {
                        date,
                        hour,
                        minute,
                        second,
                    })
                } else {
                    end.try_cast().ok_or(DateTimeError::NotRepresentable {
                        date,
                        hour,
                        minute,
                        second,
                    })
                }
            }
            // If the requested Unix time coincides with a leap second deletion, that means that we
            // cannot convert it to a valid UTC time.
            LeapSecondsResult::DeletionPoint => Err(DateTimeError::LeapSecondDeletion {
                date,
                hour,
                minute,
                second,
            }),
        }
    }
}

impl TerrestrialTimeScale for Utc {
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_datetime(1970, January, 1, 0, 0, 10)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Utc {
    type Error = LeapSecondError;

    fn try_from_time_scale<Representation, Period>(
        from: UnixTime<Representation, Period>,
    ) -> Result<UtcTime<Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<<Unix as TimeScale>::NativePeriod, <Unix as TimeScale>::NativeRepresentation>
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<<Unix as TimeScale>::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
    {
        let unix_time_seconds: UnixTime<_> = from.clone().floor();
        let subseconds = from - unix_time_seconds.clone().into_unit();
        let unix_time_seconds = unix_time_seconds.try_cast().unwrap();
        match LEAP_SECONDS.to_utc(unix_time_seconds) {
            LeapSecondsResult::Unambiguous(utc_time) => {
                Ok(utc_time.try_cast().unwrap().into_unit() + subseconds)
            }
            LeapSecondsResult::InsertionPoint { start, end } => {
                Err(LeapSecondError::Ambiguous { start, end })
            }
            LeapSecondsResult::DeletionPoint => Err(LeapSecondError::Deletion),
        }
    }
}

impl FromTimeScale<Utc> for Unix {
    fn from_time_scale<Representation, Period>(
        from: UtcTime<Representation, Period>,
    ) -> UnixTime<Representation, Period>
    where
        Period: Unit
            + FromUnit<<Utc as TimeScale>::NativePeriod, <Utc as TimeScale>::NativeRepresentation>
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<<Utc as TimeScale>::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
    {
        let utc_time_seconds: UtcTime<_> = from.clone().floor();
        let subseconds = from - utc_time_seconds.clone().into_unit();
        let utc_time_seconds = utc_time_seconds.try_cast().unwrap();
        LEAP_SECONDS
            .to_unix(utc_time_seconds)
            .try_cast()
            .unwrap()
            .into_unit()
            + subseconds
    }
}

#[derive(Debug)]
pub enum LeapSecondError {
    Ambiguous {
        start: UtcTime<i64, Second>,
        end: UtcTime<i64, Second>,
    },
    Deletion,
}

/// Describes the evolution of leap seconds over time.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct LeapSecondsTable {
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
pub(crate) enum LeapSecondsResult {
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
pub(crate) struct LeapSecondsEntry {
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

/// Two kinds of leap second changes are possible:
/// - An insertion, as has historically always been the case. Here a 61st second is added to the
///   last minute of a day.
/// - A deletion, up to now theoretical. Here, the 60th second of the last minute of a day is
///   deleted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LeapSecondsEvent {
    Insertion,
    Deletion,
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
    let regular_second1 = UtcTime::from_generic_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_generic_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_generic_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::new(2015, July, 1).unwrap();
    let regular_second3 = UtcTime::from_generic_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Leap second insertion of December 2016.
    let date = Date::new(2016, December, 31).unwrap();
    let regular_second1 = UtcTime::from_generic_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_generic_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_generic_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Seconds::new(1i64));
    assert_eq!(leap_second - regular_second1, Seconds::new(2i64));
    let date2 = Date::new(2017, January, 1).unwrap();
    let regular_second3 = UtcTime::from_generic_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Seconds::new(1i64));

    // Non-leap second date: June 2016
    let date = Date::new(2016, June, 30).unwrap();
    let regular_second1 = UtcTime::from_generic_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_generic_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Seconds::new(1i64));
    let leap_second = UtcTime::from_generic_datetime(date, 23, 59, 60);
    assert_eq!(
        leap_second,
        Err(DateTimeError::NoLeapSecondInsertion {
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
        UnixTime::from_generic_datetime(date, 23, 59, 58).unwrap(),
        UnixTime::from_generic_datetime(date, 23, 59, 59).unwrap(),
        UnixTime::from_generic_datetime(date2, 0, 0, 0).unwrap(),
        UnixTime::from_generic_datetime(date2, 0, 0, 1).unwrap(),
        UnixTime::from_generic_datetime(date3, 23, 59, 58).unwrap(),
        UnixTime::from_generic_datetime(date3, 23, 59, 59).unwrap(),
        UnixTime::from_generic_datetime(date4, 0, 0, 0).unwrap(),
        UnixTime::from_generic_datetime(date5, 23, 59, 58).unwrap(),
        UnixTime::from_generic_datetime(date5, 23, 59, 59).unwrap(),
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
                panic!("Unexpected deleted leap second found at {time:?}")
            }
        }
    }
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

/// Compares with some known timestamp values as computed manually from the Unix time and the known
/// number of leap seconds.
#[test]
fn known_leap_seconds() {
    assert_eq!(
        UtcTime::from_datetime(1970, Month::January, 1, 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(0)
    );

    assert_eq!(
        UtcTime::from_datetime(1973, Month::December, 31, 23, 59, 59)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230401)
    );

    assert_eq!(
        UtcTime::from_datetime(1973, Month::December, 31, 23, 59, 60)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230402)
    );

    assert_eq!(
        UtcTime::from_datetime(1974, Month::January, 1, 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(126230403)
    );

    assert_eq!(
        UtcTime::from_datetime(2025, Month::July, 16, 0, 0, 0)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752624027)
    );

    assert_eq!(
        UtcTime::from_datetime(2025, Month::July, 16, 17, 36, 4)
            .unwrap()
            .elapsed_time_since_epoch(),
        Seconds::new(1752687391)
    );
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    let utc = UtcTime::from_datetime(2004, Month::May, 14, 16, 43, 0).unwrap();
    let tai = TaiTime::from_datetime(2004, Month::May, 14, 16, 43, 32)
        .unwrap()
        .into_time_scale();
    assert_eq!(utc, tai);
}
