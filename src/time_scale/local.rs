//! Implementation of a "fake" time scale that is used to represent time points that are not
//! associated with an actual time scale. This is useful for representing intermediate objects.

use crate::{calendar::Datelike, duration::units::LiteralRatio, time_point::TimePoint};

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

/// A `LocalTime` represents some generic time point that has not yet been linked to a time scale.
/// Hence, it may not be compared with time points from other scales, as there is no way to link
/// them in time. It is best seen as an intermediate type that may be linked to some kind of time
/// scale identification to instantiate a "full" time point.
pub type LocalTime<Representation, Period = LiteralRatio<1>> =
    TimePoint<Local, Representation, Period>;

/// Typedef for a specialization of `LocalTime` to a unit of days. Useful as intermediate type for
/// normalization of other calendrical types.
pub type LocalDays<Representation> = LocalTime<Representation, LiteralRatio<86400, 1>>;

impl Datelike for LocalDays<i64> {}
