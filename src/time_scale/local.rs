//! Implementation of a "fake" time scale that is used to represent time points that are not
//! associated with an actual time scale. This is useful for representing intermediate objects.

use crate::{
    calendar::{Date, GregorianDate},
    duration::{Days, units::LiteralRatio},
    time_point::TimePoint,
};

/// The `Local` `TimeScale` is not actually a `TimeScale`. Instead, it is useful in scenarios where
/// some `TimePoint` may be defined, but cannot (yet) be related to an actual time scale. This is
/// useful, for example, in defining calendar arithmetic: calendrical dates are often not actually
/// defined with respect to a unique, well-specified time scale. In this manner, we can represent
/// those time points uniformly without linking them to an arbitrary time scale.
///
/// It is similar in definition and purpose to the C++ `chrono` `local_time` type. We also use the
/// Unix epoch as epoch for `LocalTime`, to make conversion between `LocalTime`, `UnixTime`, and
/// `UtcTime` a no-op cast.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Local;

pub type LocalTime<Representation, Period = LiteralRatio<1>> =
    TimePoint<Local, Representation, Period>;
pub type LocalDays<Representation> = LocalTime<Representation, LiteralRatio<86400, 1>>;

impl LocalDays<i64> {
    /// Constructs a MJD from a given historic calendar date. Applies a slight variation on the
    /// approach described by Meeus in Astronomical Algorithms (Chapter 7, Julian Day). This
    /// variation adapts the algorithm to the Unix epoch and removes the dependency on floating
    /// point arithmetic.
    pub const fn from_date(date: Date) -> Self {
        let (mut year, mut month, day) =
            (date.year() as i64, date.month() as i64, date.day() as i64);
        if month <= 2 {
            year -= 1;
            month += 12;
        }

        // Applies the leap year correction, as described in Meeus. This is needed only for
        // Gregorian dates: for dates in the Julian calendar, no such correction is needed.
        let gregorian_correction = if date.is_gregorian() {
            let a = year.div_euclid(100);
            2 - a + a / 4
        } else {
            0
        };

        // Computes the days because of elapsed years. Equivalent to `INT(365.25(Y + 4716))` from
        // Meeus.
        let year_days = (365 * (year + 4716)) + (year + 4716) / 4;

        // Computes the days due to elapsed months. Equivalent to `INT(30.6001(M + 1))` from Meeus.
        let month_days = (306001 * (month + 1)) / 10000;

        // Computes the Julian day number following Meeus' approach - though as an integer with an
        // offset of 0.5 days. Then, we subtract 2440587.5 (on top of Meeus' 1524.5) to obtain the
        // time since the Unix epoch.
        let days_since_epoch = year_days + month_days + day + gregorian_correction - 2442112;
        TimePoint::from_time_since_epoch(Days::new(days_since_epoch))
    }

    /// Constructs a MJD from a given Gregorian date. Applies a slight variation on the approach
    /// described by Meeus in Astronomical Algorithms (Chapter 7, Julian Day). This variation
    /// adapts the algorithm to the Unix epoch, and removes the dependency on floating point
    /// arithmetic.
    pub const fn from_gregorian_date(date: GregorianDate) -> Self {
        let (mut year, mut month, day) =
            (date.year() as i64, date.month() as i64, date.day() as i64);
        if month <= 2 {
            year -= 1;
            month += 12;
        }

        // Applies the leap year correction, as described in Meeus.
        let leap_year_correction = {
            let a = year.div_euclid(100);
            2 - a + a / 4
        };

        // Computes the days because of elapsed years. Equivalent to `INT(365.25(Y + 4716))` from
        // Meeus.
        let year_days = (365 * (year + 4716)) + (year + 4716) / 4;

        // Computes the days due to elapsed months. Equivalent to `INT(30.6001(M + 1))` from Meeus.
        let month_days = (306001 * (month + 1)) / 10000;

        // Computes the Julian day number following Meeus' approach - though as an integer with an
        // offset of 0.5 days. Then, we subtract 2440587.5 (on top of Meeus' 1524.5) to obtain the
        // time since the Unix epoch.
        let days_since_epoch = year_days + month_days + day + leap_year_correction - 2442112;
        TimePoint::from_time_since_epoch(Days::new(days_since_epoch))
    }
}
