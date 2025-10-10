//! Differences between two time points may be expressed as `Duration`s. These are little more than
//! a count of some unit (or multiple units) of time elapsed between those two time points. This
//! concept is similar to that applied in the C++ `chrono` library.

use core::{
    fmt::Debug,
    hash::Hash,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use num_traits::{Bounded, ConstZero, Signed, Zero};

use crate::{
    Fraction, MulCeil, MulFloor, MulRound, TryMul,
    fractional_digits::FractionalDigits,
    units::{
        Atto, Convert, Femto, Micro, Milli, Nano, Pico, Second, SecondsPerDay, SecondsPerHalfDay,
        SecondsPerHour, SecondsPerMinute, SecondsPerMonth, SecondsPerWeek, SecondsPerYear,
        TryConvert, UnitRatio,
    },
};

/// A `Duration` represents the difference between two time points. It has an associated
/// `Representation`, which determines how the count of elapsed ticks is stored. The `Period`
/// determines the integer (!) ratio of each tick to seconds. This may be used to convert between
/// `Duration`s of differing time units.
pub struct Duration<Representation, Period: ?Sized = Second> {
    count: Representation,
    period: core::marker::PhantomData<Period>,
}

impl<Representation, Period> Duration<Representation, Period>
where
    Period: ?Sized,
{
    /// Constructs a new `Duration` from a given number of time units.
    pub const fn new(count: Representation) -> Self {
        Self {
            count,
            period: core::marker::PhantomData,
        }
    }

    /// Returns the raw number of time units contained in this `Duration`. It is advised not to
    /// use this function unless absolutely necessary, as it effectively throws away all time unit
    /// information and safety.
    pub const fn count(&self) -> Representation
    where
        Representation: Copy,
    {
        self.count
    }

    /// Returns an iterator over the fractional (sub-unit) digits of this duration. Useful as
    /// helper function when printing durations.
    pub fn fractional_digits(&self, precision: usize) -> impl Iterator<Item = u8>
    where
        Representation: Copy + FractionalDigits,
        Period: UnitRatio,
    {
        self.count.fractional_digits(Period::FRACTION, precision)
    }

    /// Converts a `Duration` towards a different time unit. May only be used if the time unit is
    /// smaller than the current one (e.g., seconds to milliseconds) or if the representation of
    /// this `Duration` is a float.
    pub fn into_unit<Target>(self) -> Duration<Representation, Target>
    where
        Representation: Convert<Period, Target>,
        Target: ?Sized,
    {
        Duration::new(self.count.convert())
    }

    /// Tries to convert a `Duration` towards a different time unit. Will only return a result if
    /// the conversion is lossless.
    pub fn try_into_unit<Target>(self) -> Option<Duration<Representation, Target>>
    where
        Representation: TryConvert<Period, Target>,
        Target: ?Sized,
    {
        Some(Duration::new(self.count.try_convert()?))
    }

    /// Converts towards a different time unit, rounding towards the nearest whole unit.
    pub fn round<Target>(self) -> Duration<Representation, Target>
    where
        Representation: MulRound<Fraction, Output = Representation>,
        Target: UnitRatio + ?Sized,
        Period: UnitRatio,
    {
        let unit_ratio = Period::FRACTION.divide_by(&Target::FRACTION);
        Duration::new(self.count.mul_round(unit_ratio))
    }

    /// Converts towards a different time unit, rounding towards positive infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn ceil<Target>(self) -> Duration<Representation, Target>
    where
        Representation: MulCeil<Fraction, Output = Representation>,
        Target: UnitRatio + ?Sized,
        Period: UnitRatio,
    {
        let unit_ratio = Period::FRACTION.divide_by(&Target::FRACTION);
        Duration::new(self.count.mul_ceil(unit_ratio))
    }

    /// Converts towards a different time unit, rounding towards negative infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn floor<Target>(self) -> Duration<Representation, Target>
    where
        Representation: MulFloor<Fraction, Output = Representation>,
        Target: UnitRatio + ?Sized,
        Period: UnitRatio,
    {
        let unit_ratio = Period::FRACTION.divide_by(&Target::FRACTION);
        Duration::new(self.count.mul_floor(unit_ratio))
    }

    /// Segments this `Duration` by factoring out the largest possible number of whole multiples of
    /// a given unit. Returns this whole number as well as the remainder.
    ///
    /// An example would be factoring out the number of whole days from some elapsed time: then,
    /// `self.factor_out()` would return a tuple of the number of whole days and the fractional
    /// day part that remains.
    pub fn factor_out<Unit>(
        self,
    ) -> (
        Duration<Representation, Unit>,
        Duration<Representation, Period>,
    )
    where
        Representation: Copy
            + MulFloor<Fraction, Output = Representation>
            + Sub<Representation, Output = Representation>
            + Convert<Unit, Period>,
        Period: UnitRatio,
        Unit: UnitRatio + ?Sized,
    {
        let factored = self.floor::<Unit>();
        let remainder = self - factored.into_unit();
        (factored, remainder)
    }

    /// Infallibly converts towards a different representation.
    pub fn cast<Target>(self) -> Duration<Target, Period>
    where
        Representation: Into<Target>,
    {
        Duration::new(self.count.into())
    }

    /// Converts towards a different representation. If the underlying representation cannot store
    /// the result of this cast, returns an appropriate `Error`.
    pub fn try_cast<Target>(
        self,
    ) -> Result<Duration<Target, Period>, <Representation as TryInto<Target>>::Error>
    where
        Representation: TryInto<Target>,
    {
        Ok(Duration::new(self.count.try_into()?))
    }
}

#[cfg(kani)]
impl<Representation: kani::Arbitrary, Period> kani::Arbitrary for Duration<Representation, Period> {
    fn any() -> Self {
        Duration::new(kani::any())
    }
}

impl<Representation, Period> Debug for Duration<Representation, Period>
where
    Representation: Debug,
    Period: ?Sized,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Duration")
            .field("count", &self.count)
            .field("period", &self.period)
            .finish()
    }
}

impl<Representation, Period> Copy for Duration<Representation, Period>
where
    Representation: Copy,
    Period: ?Sized,
{
}

impl<Representation, Period> Clone for Duration<Representation, Period>
where
    Representation: Clone,
    Period: ?Sized,
{
    fn clone(&self) -> Self {
        Self::new(self.count.clone())
    }
}

impl<Representation, Period> PartialEq for Duration<Representation, Period>
where
    Representation: PartialEq,
    Period: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl<Representation, Period> Eq for Duration<Representation, Period>
where
    Representation: Eq,
    Period: ?Sized,
{
}

impl<Representation, Period> PartialOrd for Duration<Representation, Period>
where
    Representation: PartialOrd,
    Period: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.count.partial_cmp(&other.count)
    }
}

impl<Representation, Period> Ord for Duration<Representation, Period>
where
    Representation: Ord,
    Period: ?Sized,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.count.cmp(&other.count)
    }
}

impl<Representation, Period> Hash for Duration<Representation, Period>
where
    Representation: Hash,
    Period: ?Sized,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.count.hash(state);
    }
}

/// A duration that is expressed in terms of attoseconds.
pub type AttoSeconds<T> = Duration<T, Atto>;
/// A duration that is expressed in units of femtoseconds.
pub type FemtoSeconds<T> = Duration<T, Femto>;
/// A duration that is expressed in units of picoseconds.
pub type PicoSeconds<T> = Duration<T, Pico>;
/// A duration that is expressed in units of nanoseconds.
pub type NanoSeconds<T> = Duration<T, Nano>;
/// A duration that is expressed in units of microseconds.
pub type MicroSeconds<T> = Duration<T, Micro>;
/// A duration that is expressed in units of milliseconds.
pub type MilliSeconds<T> = Duration<T, Milli>;
/// A duration that is expressed in units of seconds.
pub type Seconds<T> = Duration<T, Second>;
/// A duration that is expressed in units of minutes.
pub type Minutes<T> = Duration<T, SecondsPerMinute>;
/// A duration that is expressed in units of hours.
pub type Hours<T> = Duration<T, SecondsPerHour>;
/// A duration that is expressed in units of half days.
pub type HalfDays<T> = Duration<T, SecondsPerHalfDay>;
/// A duration that is expressed in units of days.
pub type Days<T> = Duration<T, SecondsPerDay>;
/// A duration that is expressed in terms of weeks.
pub type Weeks<T> = Duration<T, SecondsPerWeek>;
/// The length of 1/12 of an average year in the Gregorian calendar.
pub type Months<T> = Duration<T, SecondsPerMonth>;
/// The length of an average year in the Gregorian calendar.
pub type Years<T> = Duration<T, SecondsPerYear>;

/// Two `Duration`s may only be added if they are of the same `Period`.
impl<R1, R2, Period> Add<Duration<R2, Period>> for Duration<R1, Period>
where
    R1: Add<R2>,
    Period: ?Sized,
{
    type Output = Duration<<R1 as Add<R2>>::Output, Period>;

    fn add(self, rhs: Duration<R2, Period>) -> Self::Output {
        Self::Output {
            count: self.count + rhs.count,
            period: core::marker::PhantomData,
        }
    }
}

impl<R1, R2, Period> AddAssign<Duration<R2, Period>> for Duration<R1, Period>
where
    R1: AddAssign<R2>,
    Period: ?Sized,
{
    fn add_assign(&mut self, rhs: Duration<R2, Period>) {
        self.count += rhs.count;
    }
}

/// Two `Duration`s may only be subtracted if they are of the same `Period`.
impl<R1, R2, Period> Sub<Duration<R2, Period>> for Duration<R1, Period>
where
    R1: Sub<R2>,
    Period: ?Sized,
{
    type Output = Duration<<R1 as Sub<R2>>::Output, Period>;

    fn sub(self, rhs: Duration<R2, Period>) -> Self::Output {
        Self::Output {
            count: self.count - rhs.count,
            period: core::marker::PhantomData,
        }
    }
}

impl<R1, R2, Period> SubAssign<Duration<R2, Period>> for Duration<R1, Period>
where
    R1: SubAssign<R2>,
    Period: ?Sized,
{
    fn sub_assign(&mut self, rhs: Duration<R2, Period>) {
        self.count -= rhs.count;
    }
}

/// A `Duration` may be negated if its `Representation` is `Signed`. This means nothing more than
/// reversing its direction in time.
impl<Representation, Period> Neg for Duration<Representation, Period>
where
    Representation: Neg<Output = Representation>,
    Period: ?Sized,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            count: self.count.neg(),
            period: core::marker::PhantomData,
        }
    }
}

impl<R1, R2, Period> Mul<R2> for Duration<R1, Period>
where
    R1: Mul<R2>,
    Period: ?Sized,
{
    type Output = Duration<<R1 as Mul<R2>>::Output, Period>;

    /// A `Duration` may not be multiplied with another `Duration` (as that is undefined), but it may
    /// be multiplied with unitless numbers.
    fn mul(self, rhs: R2) -> Self::Output {
        Self::Output {
            count: self.count * rhs,
            period: core::marker::PhantomData,
        }
    }
}

impl<R1, R2, Period> Div<R2> for Duration<R1, Period>
where
    R1: Div<R2>,
    Period: ?Sized,
{
    type Output = Duration<<R1 as Div<R2>>::Output, Period>;

    /// A `Duration` may may be divided by unitless numbers to obtain a new `Duration`.
    fn div(self, rhs: R2) -> Self::Output {
        Self::Output {
            count: self.count / rhs,
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> Bounded for Duration<Representation, Period>
where
    Representation: Bounded,
    Period: ?Sized,
{
    /// Returns the `Duration` value that is nearest to negative infinity.
    fn min_value() -> Self {
        Self {
            count: Representation::min_value(),
            period: core::marker::PhantomData,
        }
    }

    /// Returns the `Duration` value that is nearest to positive infinity.
    fn max_value() -> Self {
        Self {
            count: Representation::max_value(),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> Zero for Duration<Representation, Period>
where
    Representation: Zero,
    Period: ?Sized,
{
    /// Returns a `Duration` value that represents no time passed.
    fn zero() -> Self {
        Self {
            count: Representation::zero(),
            period: core::marker::PhantomData,
        }
    }

    /// Whether this `Duration` has any duration.
    fn is_zero(&self) -> bool {
        self.count.is_zero()
    }
}

impl<Representation, Period> ConstZero for Duration<Representation, Period>
where
    Representation: ConstZero,
    Period: ?Sized,
{
    const ZERO: Self = Self {
        count: Representation::ZERO,
        period: core::marker::PhantomData,
    };
}

impl<Representation, Period> Duration<Representation, Period>
where
    Representation: Signed,
    Period: ?Sized,
{
    pub fn abs(&self) -> Self {
        Self {
            count: self.count.abs(),
            period: core::marker::PhantomData,
        }
    }

    pub fn abs_sub(&self, other: &Self) -> Self {
        Self {
            count: self.count.abs_sub(&other.count),
            period: core::marker::PhantomData,
        }
    }

    pub fn signum(&self) -> Self {
        Self {
            count: self.count.signum(),
            period: core::marker::PhantomData,
        }
    }

    pub fn is_positive(&self) -> bool {
        self.count.is_positive()
    }

    pub fn is_negative(&self) -> bool {
        self.count.is_negative()
    }
}

impl<Representation, Period> TryMul<Fraction> for Duration<Representation, Period>
where
    Representation: TryMul<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as TryMul<Fraction>>::Output, Period>;

    fn try_mul(self, rhs: Fraction) -> Option<Self::Output> {
        Some(Duration {
            count: self.count.try_mul(rhs)?,
            period: core::marker::PhantomData,
        })
    }
}

impl<Representation, Period> TryMul<Duration<Representation, Period>> for Fraction
where
    Representation: TryMul<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as TryMul<Fraction>>::Output, Period>;

    fn try_mul(self, rhs: Duration<Representation, Period>) -> Option<Self::Output> {
        Some(Duration {
            count: rhs.count.try_mul(self)?,
            period: core::marker::PhantomData,
        })
    }
}

impl<Representation, Period> MulRound<Fraction> for Duration<Representation, Period>
where
    Representation: MulRound<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulRound<Fraction>>::Output, Period>;

    fn mul_round(self, rhs: Fraction) -> Self::Output {
        Duration {
            count: self.count.mul_round(rhs),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> MulRound<Duration<Representation, Period>> for Fraction
where
    Representation: MulRound<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulRound<Fraction>>::Output, Period>;

    fn mul_round(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Duration {
            count: rhs.count.mul_round(self),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> MulCeil<Fraction> for Duration<Representation, Period>
where
    Representation: MulCeil<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulCeil<Fraction>>::Output, Period>;

    fn mul_ceil(self, rhs: Fraction) -> Self::Output {
        Duration {
            count: self.count.mul_ceil(rhs),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> MulCeil<Duration<Representation, Period>> for Fraction
where
    Representation: MulCeil<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulCeil<Fraction>>::Output, Period>;

    fn mul_ceil(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Duration {
            count: rhs.count.mul_ceil(self),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> MulFloor<Fraction> for Duration<Representation, Period>
where
    Representation: MulFloor<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulFloor<Fraction>>::Output, Period>;

    fn mul_floor(self, rhs: Fraction) -> Self::Output {
        Duration {
            count: self.count.mul_floor(rhs),
            period: core::marker::PhantomData,
        }
    }
}

impl<Representation, Period> MulFloor<Duration<Representation, Period>> for Fraction
where
    Representation: MulFloor<Fraction>,
    Period: ?Sized,
{
    type Output = Duration<<Representation as MulFloor<Fraction>>::Output, Period>;

    fn mul_floor(self, rhs: Duration<Representation, Period>) -> Self::Output {
        Duration {
            count: rhs.count.mul_floor(self),
            period: core::marker::PhantomData,
        }
    }
}

/// Verification of the fact that conversions to SI units result in the expected ratios.
#[test]
fn convert_si_unit_seconds() {
    let seconds = Seconds::new(1.0f64);
    let milliseconds: MilliSeconds<_> = seconds.into_unit();
    assert_eq!(milliseconds.count(), 1_000.);

    let seconds = Seconds::new(1u64);
    let milliseconds: MilliSeconds<_> = seconds.into_unit();
    let microseconds: MicroSeconds<_> = seconds.into_unit();
    let nanoseconds: NanoSeconds<_> = seconds.into_unit();
    let picoseconds: PicoSeconds<_> = seconds.into_unit();
    let femtoseconds: FemtoSeconds<_> = seconds.into_unit();
    let attoseconds: AttoSeconds<_> = seconds.into_unit();

    assert_eq!(milliseconds.count(), 1_000);
    assert_eq!(microseconds.count(), 1_000_000);
    assert_eq!(nanoseconds.count(), 1_000_000_000);
    assert_eq!(picoseconds.count(), 1_000_000_000_000);
    assert_eq!(femtoseconds.count(), 1_000_000_000_000_000);
    assert_eq!(attoseconds.count(), 1_000_000_000_000_000_000);
}

/// Verification of the fact that conversions to binary fractions result in the expected ratios.
#[test]
fn convert_binary_fraction_seconds() {
    use crate::units::*;
    let seconds = Seconds::new(1u64);
    let fraction1: Duration<_, BinaryFraction1> = seconds.into_unit();
    let fraction2: Duration<_, BinaryFraction2> = seconds.into_unit();
    let fraction3: Duration<_, BinaryFraction3> = seconds.into_unit();
    let fraction4: Duration<_, BinaryFraction4> = seconds.into_unit();
    let fraction5: Duration<_, BinaryFraction5> = seconds.into_unit();
    let fraction6: Duration<_, BinaryFraction6> = seconds.into_unit();

    assert_eq!(fraction1.count(), 0x100);
    assert_eq!(fraction2.count(), 0x10000);
    assert_eq!(fraction3.count(), 0x1000000);
    assert_eq!(fraction4.count(), 0x100000000);
    assert_eq!(fraction5.count(), 0x10000000000);
    assert_eq!(fraction6.count(), 0x1000000000000);
}

/// Verification of the rounding behaviour of `Duration`s when a float is used as underlying
/// representation.
#[test]
fn rounding_floats() {
    let thirteen_hours = Hours::new(13.);
    assert_eq!(thirteen_hours.round(), Days::new(1.));

    let eleven_hours = Hours::new(11.);
    assert_eq!(eleven_hours.round(), Days::new(0.));

    let six_days = Days::new(6.);
    assert_eq!(six_days.round(), Weeks::new(1.));

    let year_fraction = Days::new(550.);
    assert_eq!(year_fraction.round(), Years::new(2.));
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::units::*;
    use paste::paste;

    /// Macro for the creation of proof harnesses that verify (formally) that a given integer
    /// roundtrip conversion will never fail, and that it will result in the original value. The
    /// only precondition is that the original value is small enough such that the multiplication
    /// does not overflow.
    macro_rules! proof_roundtrip {
        ($repr:ty, $to:ty) => {
            paste! {
                #[kani::proof]
                fn [<roundtrip_ $to:lower _ $repr:lower>]() {
                    let a: Seconds<$repr> = kani::any();
                    let numerator: Option<$repr> = <$to>::FRACTION.numerator().try_into().ok();
                    let denominator: Option<$repr> = <$to>::FRACTION.denominator().try_into().ok();

                    // We only check this conversion if the numerator and denominator can actually
                    // be represented by the target representation. It doesn't make sense, for
                    // example, to check the conversion from seconds to milliseconds for a `u8`, for
                    // example, because the conversion factor 1_000 means that a valid
                    // `Seconds<u8>` cannot be converted to a valid `MilliSeconds<u8>`.
                    if let (Some(numerator), Some(denominator)) = (numerator, denominator) {
                        kani::assume(a <= Seconds::new(<$repr>::max_value() / denominator));
                        kani::assume(a >= Seconds::new(<$repr>::min_value() / denominator));
                        let b: Duration<_, $to> = a.into_unit();
                        assert_eq!(b.count(), (a.count() * denominator) / numerator);
                        let c: Seconds<_> = b.try_into_unit().unwrap();
                        assert_eq!(a, c);
                    }
                }
            }
        };
    }

    proof_roundtrip!(u64, Atto);
    proof_roundtrip!(u64, Femto);
    proof_roundtrip!(u32, Milli);

    proof_roundtrip!(u64, BinaryFraction1);
    proof_roundtrip!(u64, BinaryFraction2);
    proof_roundtrip!(u64, BinaryFraction3);
    proof_roundtrip!(u64, BinaryFraction4);
    proof_roundtrip!(u64, BinaryFraction5);
    proof_roundtrip!(u64, BinaryFraction6);

    proof_roundtrip!(i64, BinaryFraction1);
    proof_roundtrip!(i64, BinaryFraction2);
    proof_roundtrip!(i64, BinaryFraction3);
    proof_roundtrip!(i64, BinaryFraction4);
    proof_roundtrip!(i64, BinaryFraction5);
    proof_roundtrip!(i64, BinaryFraction6);

    /// Macro for the creation of proof harnesses that verify (formally) that a given integer
    /// rounding will never fail. The only precondition is that the number is small enough not to
    /// result in overflow when multiplied with the unit conversion factor.
    macro_rules! proof_rounding {
        ($repr:ty, $to:ty) => {
            paste! {
                #[kani::proof]
                fn [<rounding_ $to:lower _ $repr:lower>]() {
                    let a: Seconds<$repr> = kani::any();
                    let numerator: Option<$repr> = <$to>::FRACTION.numerator().try_into().ok();
                    let denominator: Option<$repr> = <$to>::FRACTION.denominator().try_into().ok();

                    // We only check this conversion if the numerator and denominator can actually
                    // be represented by the target representation. It doesn't make sense, for
                    // example, to check the conversion from seconds to milliseconds for a `u8`, for
                    // example, because the conversion factor 1_000 means that a valid
                    // `Seconds<u8>` cannot be converted to a valid `MilliSeconds<u8>`.
                    if let (Some(_), Some(denominator)) = (numerator, denominator) {
                        kani::assume(a < Seconds::new(<$repr>::max_value() / denominator));
                        kani::assume(a > Seconds::new(<$repr>::min_value() / denominator));
                        let _: Duration<_, $to> = a.round();
                        let _: Duration<_, $to> = a.ceil();
                        let _: Duration<_, $to> = a.floor();
                    }
                }
            }
        };
    }

    /// Repeats the rounding proof harness for all primitive integer types.
    macro_rules! proof_rounding_all_integers {
        ($to:ty) => {
            proof_rounding!(u128, $to);
            proof_rounding!(u64, $to);
            proof_rounding!(u32, $to);
            proof_rounding!(u16, $to);
            proof_rounding!(u8, $to);
            proof_rounding!(i128, $to);
            proof_rounding!(i64, $to);
            proof_rounding!(i32, $to);
            proof_rounding!(i16, $to);
            proof_rounding!(i8, $to);
        };
    }

    proof_rounding_all_integers!(Atto);
    proof_rounding_all_integers!(Femto);
    proof_rounding_all_integers!(Pico);
    proof_rounding_all_integers!(Nano);
    proof_rounding_all_integers!(Micro);
    proof_rounding_all_integers!(Milli);
    proof_rounding_all_integers!(Centi);
    proof_rounding_all_integers!(BinaryFraction1);
    proof_rounding_all_integers!(BinaryFraction2);
    proof_rounding_all_integers!(BinaryFraction3);
    proof_rounding_all_integers!(BinaryFraction4);
    proof_rounding_all_integers!(BinaryFraction5);
    proof_rounding_all_integers!(BinaryFraction6);
    proof_rounding_all_integers!(SecondsPerMinute);
    proof_rounding_all_integers!(SecondsPerHour);
    proof_rounding_all_integers!(SecondsPerDay);
    proof_rounding_all_integers!(SecondsPerWeek);
    proof_rounding_all_integers!(SecondsPerMonth);
    proof_rounding_all_integers!(SecondsPerYear);
}

/// Verifies that integer rounding logic is implemented correctly for some known values.
#[test]
fn rounding_integers() {
    let thirteen_hours = Hours::new(13);
    assert_eq!(thirteen_hours.round(), Days::new(1));

    let eleven_hours = Hours::new(11);
    assert_eq!(eleven_hours.round(), Days::new(0));

    let six_days = Days::new(6);
    assert_eq!(six_days.round(), Weeks::new(1));

    let year_fraction = Days::new(550);
    assert_eq!(year_fraction.round(), Years::new(2));

    let seconds_per_minute = Seconds::new(-99i8);
    assert_eq!(seconds_per_minute.round(), Minutes::new(-2));
}
