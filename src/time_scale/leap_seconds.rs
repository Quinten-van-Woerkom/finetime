//! Leap seconds are applied when converting date-time pairs to underlying time scales, to better
//! align those time scales with the human-centric time based on the Earth's rotation (UT1).

use crate::{Date, FromDateTime, IntoDateTime, Seconds};

/// Since leap seconds are hard to predict in advance (due to irregular variations in the Earth's
/// rotation), their insertion and deletion is based on short-term predictions. This means that
/// it is not possible to develop a leap second table that holds "for all eternity" without
/// external influence. Different applications may have different manners of obtaining these
/// updates from external sources - if at all possible. To accommodate all such applications, we
/// support a generic manner of introducing leap seconds, via the `LeapSecondProvider` interface.
///
/// Any type that implements this trait may be used to determine when leap seconds occur, and how
/// often they do. In this manner, one may opt for a static leap second table but also easily swap
/// it for a table that updates based on the published IANA list, on GNSS constellation navigation
/// messages, or custom telecommands (for spacecraft, for example).
pub trait LeapSecondProvider {
    /// For any given date (expressed in UTC), determines whether a leap second was inserted at the
    /// end of that day. In tandem, returns the accumulated number of leap seconds before (!) that
    /// date.
    fn leap_seconds_on_date(&self, utc_date: Date<i32>) -> (bool, Seconds<u8>);
}

/// This trait is the leap second equivalent of `FromDateTime`. It permits the creation of time
/// points from date-times when a non-standard leap second provider must be used.
pub trait FromLeapSecondDateTime: FromDateTime {
    /// Maps a given combination of date and time-of-day to an instant on this time scale. May
    /// return an error if the input does not represent a valid combination of date and
    /// time-of-day.
    ///
    /// Takes a leap second provider as additional argument, which is used to determine at which
    /// times leap seconds are inserted or deleted.
    fn from_leap_second_datetime(
        date: Date<i32>,
        hour: u8,
        minute: u8,
        second: u8,
        leap_second_provider: &impl LeapSecondProvider,
    ) -> Result<Self, Self::Error>;
}

/// This trait is the leap second equivalent of `IntoDateTime`. It permits the retrieval of
/// date-times from time points when a non-standard leap second provider must be used.
pub trait IntoLeapSecondDateTime: IntoDateTime {
    /// Maps a time point back to the date and time-of-day that it represents. Returns a tuple of
    /// date, hour, minute, and second. This function shall not fail, unless overflow occurs in the
    /// underlying integer arithmetic.
    ///
    /// Takes a leap second provider as additional argument, which is used to determine at which
    /// times leap seconds are inserted or deleted.
    fn into_datetime(
        self,
        leap_second_provider: &impl LeapSecondProvider,
    ) -> (Date<i32>, u8, u8, u8);
}

/// Default leap second provider that uses a pre-compiled table to obtain the leap seconds. Will
/// suffice for most non-critical applications and is useful in testing, but cannot be updated
/// after compilation. This makes it unsuitable for long-running applications.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct StaticLeapSecondProvider {}

/// Convenience constant that may be used to directly obtain a `StaticLeapSecondProvider` object.
pub const STATIC_LEAP_SECOND_PROVIDER: StaticLeapSecondProvider = StaticLeapSecondProvider {};

impl LeapSecondProvider for StaticLeapSecondProvider {
    /// For the static leap seconds provider, we just use a generated jump table that maps from
    /// days (expressed as `Date<i32>`, i.e., `Days<i32>` since 1970-01-01) to whether that day
    /// contains a leap second and what the total leap second count is. It is sorted in reverse,
    /// because it is more likely for users to work with dates in the present or future than in the
    /// past.
    fn leap_seconds_on_date(&self, utc_date: Date<i32>) -> (bool, Seconds<u8>) {
        let days_since_1970_01_01 = utc_date.time_since_epoch().count();
        let (is_leap_second, leap_seconds) = match days_since_1970_01_01 {
            17167.. => (false, 37),
            17166 => (true, 36),
            16617.. => (false, 36),
            16616 => (true, 35),
            15522.. => (false, 35),
            15521 => (true, 34),
            14245.. => (false, 34),
            14244 => (true, 33),
            13149.. => (false, 33),
            13148 => (true, 32),
            10592.. => (false, 32),
            10591 => (true, 31),
            10043.. => (false, 31),
            10042 => (true, 30),
            9496.. => (false, 30),
            9495 => (true, 29),
            8947.. => (false, 29),
            8946 => (true, 28),
            8582.. => (false, 28),
            8581 => (true, 27),
            8217.. => (false, 27),
            8216 => (true, 26),
            7670.. => (false, 26),
            7669 => (true, 25),
            7305.. => (false, 25),
            7304 => (true, 24),
            6574.. => (false, 24),
            6573 => (true, 23),
            5660.. => (false, 23),
            5659 => (true, 22),
            4929.. => (false, 22),
            4928 => (true, 21),
            4564.. => (false, 21),
            4563 => (true, 20),
            4199.. => (false, 20),
            4198 => (true, 19),
            3652.. => (false, 19),
            3651 => (true, 18),
            3287.. => (false, 18),
            3286 => (true, 17),
            2922.. => (false, 17),
            2921 => (true, 16),
            2557.. => (false, 16),
            2556 => (true, 15),
            2191.. => (false, 15),
            2190 => (true, 14),
            1826.. => (false, 14),
            1825 => (true, 13),
            1461.. => (false, 13),
            1460 => (true, 12),
            1096.. => (false, 12),
            1095 => (true, 11),
            912.. => (false, 11),
            911 => (true, 10),
            730.. => (false, 10),
            729 => (true, 9),
            _ => (false, 9),
        };
        (is_leap_second, Seconds::new(leap_seconds))
    }
}

// #[cfg(test)]
// fn print_date(year: i32, month: crate::Month, day: u8, leap_seconds: u8) {
//     let day_count = Date::from_historic_date(year, month, day)
//         .unwrap()
//         .time_since_epoch()
//         .count();
//     println!("{day_count}.. => (false, {leap_seconds}),");
//     println!("{} => (true, {}),", day_count - 1, leap_seconds - 1);
// }

// #[test]
// fn print_dates() {
//     print_date(2017, crate::Month::January, 1, 37);
//     print_date(2015, crate::Month::July, 1, 36);
//     print_date(2012, crate::Month::July, 1, 35);
//     print_date(2009, crate::Month::January, 1, 34);
//     print_date(2006, crate::Month::January, 1, 33);
//     print_date(1999, crate::Month::January, 1, 32);
//     print_date(1997, crate::Month::July, 1, 31);
//     print_date(1996, crate::Month::January, 1, 30);
//     print_date(1994, crate::Month::July, 1, 29);
//     print_date(1993, crate::Month::July, 1, 28);
//     print_date(1992, crate::Month::July, 1, 27);
//     print_date(1991, crate::Month::January, 1, 26);
//     print_date(1990, crate::Month::January, 1, 25);
//     print_date(1988, crate::Month::January, 1, 24);
//     print_date(1985, crate::Month::July, 1, 23);
//     print_date(1983, crate::Month::July, 1, 22);
//     print_date(1982, crate::Month::July, 1, 21);
//     print_date(1981, crate::Month::July, 1, 20);
//     print_date(1980, crate::Month::January, 1, 19);
//     print_date(1979, crate::Month::January, 1, 18);
//     print_date(1978, crate::Month::January, 1, 17);
//     print_date(1977, crate::Month::January, 1, 16);
//     print_date(1976, crate::Month::January, 1, 15);
//     print_date(1975, crate::Month::January, 1, 14);
//     print_date(1974, crate::Month::January, 1, 13);
//     print_date(1973, crate::Month::January, 1, 12);
//     print_date(1972, crate::Month::July, 1, 11);
//     print_date(1972, crate::Month::January, 1, 10);

//     assert!(false);
// }
