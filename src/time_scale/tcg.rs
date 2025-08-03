//! Implementation of the Geocentric Coordinate Time (TCG) time scale.

use crate::{
    Date, FromTimeScale, LocalTime, MilliSeconds, Month, TimePoint, TimeScale, Tt, TtTime,
    arithmetic::{Fraction, FromUnit, Milli, Second, TimeRepresentation, TryFromExact, Unit},
};

/// `TcgTime` is a specialization of `TimePoint` that uses the Geocentric Coordinate Time (TCG)
/// time scale.
pub type TcgTime<Representation, Period = Second> = TimePoint<Tcg, Representation, Period>;

/// The Geocentric Coordinate Time (TCG) is the time of a hypothetical clock that is placed at the
/// center of the non-rotating geocentric reference frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tcg;

impl TimeScale for Tcg {
    type NativePeriod = Milli;

    type NativeRepresentation = i128;

    /// For practical reasons (conversion to and from TT), it is convenient to set the TCG epoch to
    /// 1977-01-01T00:00:32.184: at this time, TT and TCG match exactly (by definition).
    fn epoch() -> LocalTime<Self::NativeRepresentation, Self::NativePeriod> {
        let date = Date::new(1977, Month::January, 1).unwrap();
        let epoch = date.to_local_days().into_unit() + MilliSeconds::new(32_184);
        epoch.try_cast().unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl FromTimeScale<Tcg> for Tt {
    fn from_time_scale<Representation, Period>(
        from: TimePoint<Tcg, Representation, Period>,
    ) -> TimePoint<Tt, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation
            + TryFromExact<<Tcg as TimeScale>::NativeRepresentation>
            + TryFromExact<<Tt as TimeScale>::NativeRepresentation>,
        Period:
            FromUnit<<Tcg as TimeScale>::NativePeriod, <Tcg as TimeScale>::NativeRepresentation>,
        Period: FromUnit<<Tt as TimeScale>::NativePeriod, <Tt as TimeScale>::NativeRepresentation>,
    {
        // We encode the conversion factor (= (1.0 - 6.969290134e-10)) as an exact fraction, such
        // that integer arithmetic can be done to exact precision, even when some rounding is
        // needed at the end of the conversion. This exactness is warranted by the fact that this
        // is a defining constant: hence, analytic exactness is actually meaningful and not an
        // approximation.
        const CONVERSION_FACTOR: Fraction = Fraction::new(4999999996515354933, 5000000000000000000);
        let tcg_time_since_epoch = from.elapsed_time_since_epoch();
        let tt_time_since_epoch = tcg_time_since_epoch.multiply_fraction(CONVERSION_FACTOR);
        TtTime::from_time_since_epoch(tt_time_since_epoch)
    }
}

impl FromTimeScale<Tt> for Tcg {
    fn from_time_scale<Representation, Period>(
        from: TimePoint<Tt, Representation, Period>,
    ) -> TimePoint<Tcg, Representation, Period>
    where
        Period: Unit,
        Representation: TimeRepresentation
            + TryFromExact<<Tt as TimeScale>::NativeRepresentation>
            + TryFromExact<Self::NativeRepresentation>,
        Period: FromUnit<<Tt as TimeScale>::NativePeriod, <Tt as TimeScale>::NativeRepresentation>,
        Period: FromUnit<Self::NativePeriod, Self::NativeRepresentation>,
    {
        // We encode the conversion factor (= (1.0 - 6.969290134e-10)) as an exact fraction, such
        // that integer arithmetic can be done to exact precision, even when some rounding is
        // needed at the end of the conversion. This exactness is warranted by the fact that this
        // is a defining constant: hence, analytic exactness is actually meaningful and not an
        // approximation.
        const CONVERSION_FACTOR: Fraction = Fraction::new(5000000000000000000, 4999999996515354933);
        let tt_time_since_epoch = from.elapsed_time_since_epoch();
        let tcg_time_since_epoch = tt_time_since_epoch.multiply_fraction(CONVERSION_FACTOR);
        TcgTime::from_time_since_epoch(tcg_time_since_epoch)
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TCG time from a historic date and time stamp never panics.
    /// An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TcgTime::from_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that construction of a TCG time from a Gregorian date and time stamp never panics.
    /// An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        let year: i32 = kani::any();
        let month: Month = kani::any();
        let day: u8 = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TcgTime::from_gregorian_datetime(year, month, day, hour, minute, second);
    }

    /// Verifies that all valid TCG time datetimes can be converted to and from the equivalent TT
    /// time without panics. An assumption is made on the input range because some dates result
    /// in a count of milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn datetime_tt_tcg_roundtrip() {
        let date: Date = kani::any();
        let year: i32 = date.year();
        let month: Month = date.month();
        let day: u8 = date.day();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1: TcgTime<i128, crate::arithmetic::Nano> =
            TcgTime::from_datetime(year, month, day, hour, minute, second)
                .unwrap()
                .cast()
                .into_unit();
        let tt: TtTime<_, _> = time1.into_time_scale();
        let _: TcgTime<_, _> = tt.into_time_scale();
    }
}

/// Verifies the TT-TCG conversion using some known values.
#[test]
fn datetime_tt_tcg_conversion() {
    use crate::Month::*;
    use crate::arithmetic::{Atto, Micro, Pico};
    use crate::{MicroSeconds, NanoSeconds, Seconds};

    // At the epoch 1977-01-01T00:00:32.184, both time stamps should be exactly equivalent. We
    // check this to attosecond precision, because there should be no overflow anyway at the epoch.
    let time1 = TcgTime::from_subsecond_generic_datetime(
        Date::new(1977, January, 1).unwrap(),
        0,
        0,
        32,
        MilliSeconds::new(184i64),
    )
    .unwrap()
    .into_unit::<Atto>();
    let time2 = TtTime::from_subsecond_generic_datetime(
        Date::new(1977, January, 1).unwrap(),
        0,
        0,
        32,
        MilliSeconds::new(184i64),
    )
    .unwrap()
    .into_unit::<Atto>();
    assert_eq!(time1, time2.into_time_scale());

    // 10_000_000_000 seconds after that epoch, there should be a difference of 6.969290134 seconds
    // based on the known rate difference of L_G = 6.969290134e-10. We check this to picosecond
    // precision: the offset shall be exactly 6.969290134000 seconds (to picosecond accuracy),
    // since this rate difference is a defining constant (not just an approximation).
    let time1 = time1.cast::<i128>().round::<Pico>() + Seconds::new(10_000_000_000i128).into_unit();
    let time2 = time2.cast::<i128>().round::<Pico>() + Seconds::new(10_000_000_000i128).into_unit()
        - NanoSeconds::new(6_969_290_134i128).into_unit();
    assert_eq!(time1.into_time_scale(), time2);

    // At J2000, the difference should be about 505.833 ms (see "Report of the IAU WGAS Sub-group
    // on Issues on Time", P.K. Seidelmann).
    let time1 = TtTime::from_datetime(2000, January, 1, 12, 0, 0)
        .unwrap()
        .into_unit::<Micro>();
    let time2 = TcgTime::from_subsecond_generic_datetime(
        Date::new(2000, January, 1).unwrap(),
        12,
        0,
        0,
        MicroSeconds::new(505_833),
    )
    .unwrap()
    .into_unit();
    assert_eq!(time1.into_time_scale(), time2);

    // At J2100 (2100-01-01T12:00:00 TT), the difference should be 2.70517411 seconds (see "Report
    // of the IAU WGAS Sub-group on Issues on Time", P.K. Seidelmann). Redoing the math using
    // exact arithmetic leads to an expected result of 2.705173778 seconds (which is also our
    // result), so we only check this to microsecond precision.
    let time1 = TtTime::from_datetime(2100, January, 1, 12, 0, 0)
        .unwrap()
        .into_unit::<Micro>();
    let time2 = TcgTime::from_subsecond_generic_datetime(
        Date::new(2100, January, 1).unwrap(),
        12,
        0,
        2,
        NanoSeconds::new(705_174_110),
    )
    .unwrap()
    .round::<Micro>();
    assert_eq!(time1.into_time_scale(), time2);
}
