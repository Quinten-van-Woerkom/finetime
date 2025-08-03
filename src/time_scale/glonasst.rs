//! Implementation of the GLONASS Time (GLONASST) time scale.

use crate::{
    DateTimeError, Days, LeapSecondError, LocalTime, TaiTime, TerrestrialTimeScale, TimePoint,
    TimeScale, TryFromTimeScale, Unix, Utc,
    arithmetic::{
        FromUnit, Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute, TimeRepresentation,
        TryFromExact, Unit,
    },
    calendar::{Date, Month},
};

/// `GlonassTime` is a time point that is expressed according to the GLONASS Time time
/// scale.
pub type GlonassTime<Representation, Period = Second> = TimePoint<Glonasst, Representation, Period>;

/// The GLONASS Time (GLONASST) time scale is broadcast by GLONASS satellites. It follows UTC (or
/// rather, UTC(SU), which is a realization of UTC) and adds three hours (Moscow time). Indeed,
/// this means that it also incorporates leap seconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Glonasst;

impl TimeScale for Glonasst {
    type NativePeriod = Second;

    type NativeRepresentation = i64;

    /// The GLONASS epoch is 1996-01-01T00:00:00 UTC(SU). However, the GLONASS broadcast time is
    /// offset by +3h from UTC(SU) to match Moscow time. This means that its broadcast time (MSK)
    /// would have been 0 at 1996-01-01T00:00:00 MSK, which is what we define as epoch.
    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        Date::new(1996, Month::January, 1)
            .unwrap()
            .to_local_days()
            .into_unit()
            .try_cast()
            .unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }

    fn from_local_datetime(
        date: super::LocalDays<i64>,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<TimePoint<Self, Self::NativeRepresentation, Self::NativePeriod>, DateTimeError>
    where
        Self::NativePeriod: FromUnit<SecondsPerDay, Self::NativeRepresentation>
            + FromUnit<SecondsPerHour, Self::NativeRepresentation>
            + FromUnit<SecondsPerMinute, Self::NativeRepresentation>
            + FromUnit<Second, Self::NativeRepresentation>,
        Second: FromUnit<SecondsPerDay, Self::NativeRepresentation>
            + FromUnit<SecondsPerHour, Self::NativeRepresentation>
            + FromUnit<SecondsPerMinute, Self::NativeRepresentation>
            + FromUnit<Second, Self::NativeRepresentation>,
    {
        // First, we convert the requested datetime into the equivalent datetime in UTC. This is
        // quite simple, because GLONASST is always exactly 3 hours ahead of UTC(SU), the
        // realization of UTC by the Russian Federation.
        let (date_utc, hour_utc) = {
            let wrapped_hour = hour.wrapping_sub(3);
            // Check whether we crossed a date boundary. Note that we check for <21 instead of <24
            // to ensure that input times with hours of 24+ result in errors in the
            // `Utc::from_local_datetime()` call.
            if wrapped_hour < 21 {
                // No wrapping occured
                (date, wrapped_hour)
            } else {
                // Wrapping occured, so we correct by removing one day from the date
                (date - Days::new(1), wrapped_hour.wrapping_add(24))
            }
        };

        // Finally, we may create the requested UTC time that corresponds with the requested
        // GLONASS time. However, if we error, we must patch the errors to update them with the
        // correct requested local datetimes.
        let utc_time = match Utc::from_local_datetime(date_utc, hour_utc, minute, second) {
            Ok(utc_time) => utc_time,
            Err(error) => {
                return Err(match error {
                    DateTimeError::InvalidTimeOfDay { .. } => DateTimeError::InvalidTimeOfDay {
                        hour,
                        minute,
                        second,
                    },
                    DateTimeError::NoLeapSecondInsertion { .. } => {
                        DateTimeError::NoLeapSecondInsertion {
                            date,
                            hour,
                            minute,
                            second,
                        }
                    }
                    DateTimeError::LeapSecondDeletion { .. } => DateTimeError::LeapSecondDeletion {
                        date,
                        hour,
                        minute,
                        second,
                    },
                    DateTimeError::NotRepresentable { .. } => DateTimeError::NotRepresentable {
                        date,
                        hour,
                        minute,
                        second,
                    },
                    DateTimeError::InvalidHistoricDate { .. } => unreachable!(),
                    DateTimeError::InvalidGregorianDate { .. } => unreachable!(),
                    DateTimeError::InvalidDayOfYear { .. } => unreachable!(),
                });
            }
        };

        // Finally, we may convert the resulting datetime back to GLONASST by subtracting the
        // difference between the time scale epochs.
        let seconds_since_utc = utc_time.elapsed_time_since_epoch();
        let glonasst_epoch = Self::epoch_tai();
        let utc_epoch = Utc::epoch_tai();
        let epoch_difference = glonasst_epoch - utc_epoch;
        let seconds_since_glonasst = seconds_since_utc - epoch_difference;
        Ok(GlonassTime::from_time_since_epoch(seconds_since_glonasst))
    }
}

impl TerrestrialTimeScale for Glonasst {
    fn epoch_tai() -> TaiTime<Self::NativeRepresentation, Self::NativePeriod> {
        TaiTime::from_datetime(1995, Month::December, 31, 21, 0, 29)
            .unwrap()
            .into_unit()
            .try_cast()
            .unwrap()
    }
}

impl TryFromTimeScale<Unix> for Glonasst {
    type Error = LeapSecondError;

    fn try_from_time_scale<Representation, Period>(
        from: TimePoint<Unix, Representation, Period>,
    ) -> Result<TimePoint<Self, Representation, Period>, Self::Error>
    where
        Period: Unit
            + FromUnit<<Unix as TimeScale>::NativePeriod, <Unix as TimeScale>::NativeRepresentation>
            + FromUnit<Self::NativePeriod, Self::NativeRepresentation>
            + FromUnit<Second, Representation>,
        Representation: TimeRepresentation
            + TryFromExact<<Unix as TimeScale>::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
    {
        let utc_time = Utc::try_from_time_scale(from)?;
        Ok(utc_time.into_time_scale())
    }
}

/// Compares with a known timestamp as obtained from the definition of the GLONASS time: the
/// epoch itself of the system. For GLONASST, two times could be considered as epoch:
/// 1996-01-01T00:00:00 UTC(SU), which is defined as start of the time scale, and
/// 1996-01-01T00:00:00 MSK, which is the epoch at which the broadcast time is 0. We just check
/// both times, and we also verify that the second is really the zero-duration point of this type.
#[test]
fn known_timestamps() {
    use crate::{Seconds, UtcTime};
    let utc = UtcTime::from_datetime(1996, Month::January, 1, 0, 0, 0).unwrap();
    let glonasst = GlonassTime::from_datetime(1996, Month::January, 1, 3, 0, 0).unwrap();
    assert_eq!(utc, glonasst.into_time_scale());

    let utc = UtcTime::from_datetime(1995, Month::December, 31, 21, 0, 0).unwrap();
    let glonasst = GlonassTime::from_datetime(1996, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(utc, glonasst.into_time_scale());
    assert_eq!(glonasst.elapsed_time_since_epoch(), Seconds::new(0));
}
