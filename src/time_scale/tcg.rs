//! Implementation of the Geocentric Coordinate Time (TCG) time scale.

use num::{NumCast, traits::NumOps};

use crate::{
    Date, LocalTime, MilliSeconds, Month, TaiTime, TimePoint, TimeScale, TimeScaleConversion, Tt,
    TtTime,
    units::{IsValidConversion, LiteralRatio, Milli, Ratio},
};

/// `TcgTime` is a specialization of `TimePoint` that uses the Geocentric Coordinate Time (TCG)
/// time scale.
pub type TcgTime<Representation, Period = LiteralRatio<1>> = TimePoint<Tcg, Representation, Period>;

/// The Geocentric Coordinate Time (TCG) is the time of a hypothetical clock that is placed at the
/// center of the non-rotating geocentric reference frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tcg;

impl TimeScale for Tcg {
    type NativePeriod = Milli;

    /// At its epoch, TCG is exactly (by definition) 32.184 seconds ahead of TAI. This means that
    /// its epoch is precisely 1977-01-01T00:00:00 TAI.
    fn epoch_tai<T>() -> TaiTime<T, Self::NativePeriod>
    where
        T: NumCast,
    {
        let date = Date::new(1977, Month::January, 1).unwrap();
        TaiTime::from_datetime(date, 0, 0, 0)
            .unwrap()
            .convert()
            .try_cast()
            .unwrap()
    }

    /// For practical reasons (conversion to and from TT), it is convenient to set the TCG epoch to
    /// 1977-01-01T00:00:32.184: at this time, TT and TCG match exactly (by definition).
    fn epoch_local<T>() -> LocalTime<T, Self::NativePeriod>
    where
        T: NumCast,
    {
        let date = Date::new(1977, Month::January, 1).unwrap();
        let epoch = date.to_local_days().convert() + MilliSeconds::new(32_184);
        epoch.try_cast().unwrap()
    }

    fn counts_leap_seconds() -> bool {
        false
    }
}

impl TimeScaleConversion<Tcg, Tt> for () {
    fn transform<Representation, Period>(
        from: TimePoint<Tcg, Representation, Period>,
    ) -> TimePoint<Tt, Representation, Period>
    where
        Period: Ratio,
        Representation: Copy + NumCast + NumOps,
        (): IsValidConversion<i64, <Tcg as TimeScale>::NativePeriod, Period>
            + IsValidConversion<i64, <Tt as TimeScale>::NativePeriod, Period>,
    {
        // We implement the underlying rate transformation in double-precision arithmetic. This may
        // introduce some floating point error, but the alternative is to try and implement exact
        // integer division, which would significantly complexify the procedure.
        let tcg_time_since_epoch = from.elapsed_time_since_epoch();
        const CONVERSION_FACTOR: f64 = -6.969290134e-10;
        let tt_time_since_epoch =
            tcg_time_since_epoch + tcg_time_since_epoch.multiply_float(CONVERSION_FACTOR);
        TtTime::from_time_since_epoch(tt_time_since_epoch)
    }
}

impl TimeScaleConversion<Tt, Tcg> for () {
    fn transform<Representation, Period>(
        from: TimePoint<Tt, Representation, Period>,
    ) -> TimePoint<Tcg, Representation, Period>
    where
        Period: Ratio,
        Representation: Copy + NumCast + NumOps,
        (): IsValidConversion<i64, <Tt as TimeScale>::NativePeriod, Period>
            + IsValidConversion<i64, <Tcg as TimeScale>::NativePeriod, Period>,
    {
        // We implement the underlying rate transformation in double-precision arithmetic. This may
        // introduce some floating point error, but the alternative is to try and implement exact
        // integer division, which would significantly complexify the procedure.
        let tt_time_since_epoch = from.elapsed_time_since_epoch();
        const CONVERSION_FACTOR: f64 = 6.969290134e-10 / (1.0 - 6.969290134e-10);
        let tcg_time_since_epoch =
            tt_time_since_epoch + tt_time_since_epoch.multiply_float(CONVERSION_FACTOR);
        TcgTime::from_time_since_epoch(tcg_time_since_epoch)
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::Date;

    /// Verifies that construction of a TCG time from a historic date and time stamp never panics.
    /// An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_datetime_never_panics() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > Date::new(i32::MIN / 8, Month::January, 1).unwrap());
        kani::assume(date < Date::new(i32::MAX / 8, Month::December, 31).unwrap());
        let _ = TcgTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that construction of a TCG time from a Gregorian date and time stamp never panics.
    /// An assumption is made on the input range because some dates result in a count of
    /// milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn from_gregorian_never_panics() {
        use crate::calendar::GregorianDate;
        let date: GregorianDate = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > GregorianDate::new(i32::MIN / 8, Month::January, 1).unwrap());
        kani::assume(date < GregorianDate::new(i32::MAX / 8, Month::December, 31).unwrap());
        let _ = TcgTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid TCG time datetimes can be converted to and from the equivalent TT
    /// time without panics. An assumption is made on the input range because some dates result
    /// in a count of milliseconds from the TCG epoch that is too large to store in an `i64`.
    #[kani::proof]
    fn datetime_tt_tcg_roundtrip() {
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(date > Date::new(i32::MIN / 32, Month::January, 1).unwrap());
        kani::assume(date < Date::new(i32::MAX / 32, Month::December, 31).unwrap());
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1: TcgTime<i128, crate::units::Nano> =
            TcgTime::from_datetime(date, hour, minute, second)
                .unwrap()
                .cast()
                .convert();
        let tt: TtTime<_, _> = time1.transform();
        let _: TcgTime<_, _> = tt.transform();
    }
}

/// Verifies the TT-TCG conversion using some known values.
#[test]
fn datetime_tt_tcg_conversion() {
    // At the epoch 1977-01-01T00:00:32.184, both time stamps should be exactly equivalent.
    let time1 = TcgTime::from_subsecond_datetime(
        Date::new(1977, Month::January, 1).unwrap(),
        0,
        0,
        32,
        MilliSeconds::new(184),
    )
    .unwrap();
    let time2 = TtTime::from_subsecond_datetime(
        Date::new(1977, Month::January, 1).unwrap(),
        0,
        0,
        32,
        MilliSeconds::new(184),
    )
    .unwrap();
    assert_eq!(time1, time2.transform());

    // 10_000_000_000 seconds after that epoch, there should be a difference of 6.969290134 seconds
    use crate::{MicroSeconds, Seconds};
    let time1 = time1.convert() + Seconds::new(10_000_000_000i64).convert();
    let time2 =
        time2.convert() + Seconds::new(10_000_000_000i64).convert() - MicroSeconds::new(6969290i64);
    assert_eq!(time1.transform(), time2);

    // At J2000, the difference should be about 505 ms.
    let time1 = TtTime::from_datetime(Date::new(2000, Month::January, 1).unwrap(), 12, 0, 0)
        .unwrap()
        .convert::<crate::units::Micro>();
    let time2 = TcgTime::from_subsecond_datetime(
        Date::new(2000, Month::January, 1).unwrap(),
        12,
        0,
        0,
        crate::MicroSeconds::new(505833),
    )
    .unwrap()
    .convert();
    assert_eq!(time1.transform(), time2);
}
