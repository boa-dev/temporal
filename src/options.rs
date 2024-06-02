//! Native implementation of the `Temporal` options.
//!
//! Temporal has various instances where user's can define options for how an
//! operation may be completed.

use core::{fmt, num::NonZeroU32, str::FromStr};

use crate::{
    components::{calendar::CalendarProtocol, tz::TzProtocol, Date, ZonedDateTime},
    TemporalError, TemporalResult,
};

// ==== RoundingIncrement option ====
// Invariants:
// RoundingIncrement(n): 1 <= n < 10^9
/// A numeric rounding increment.
// TODO: Add explanation on what exactly are rounding increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoundingIncrement(pub(crate) NonZeroU32);

impl Default for RoundingIncrement {
    fn default() -> Self {
        Self::ONE
    }
}

impl TryFrom<f64> for RoundingIncrement {
    type Error = TemporalError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // GetRoundingIncrementOption ( options )
        // https://tc39.es/proposal-temporal/#sec-temporal-getroundingincrementoption

        // 4. If increment is not finite, throw a RangeError exception.
        if !value.is_finite() {
            return Err(TemporalError::range().with_message("roundingIncrement must be finite"));
        }

        // 5. Let integerIncrement be truncate(ℝ(increment)).
        let integer_increment = value.trunc();
        // 6. If integerIncrement < 1 or integerIncrement > 10**9, throw a RangeError exception.
        if !(1.0..=1_000_000_000.0).contains(&integer_increment) {
            return Err(TemporalError::range()
                .with_message("roundingIncrement cannot be less that 1 or bigger than 10**9"));
        }
        // 7. Return integerIncrement.
        // SAFETY: Check above guarantees that `integer_increment` is within the bounds of
        // NonZeroU32.
        unsafe { Ok(Self::new_unchecked(integer_increment as u32)) }
    }
}

impl RoundingIncrement {
    // Using `MIN` avoids either a panic or using NonZeroU32::new_unchecked
    /// A rounding increment of 1 (normal rounding).
    pub const ONE: Self = Self(NonZeroU32::MIN);

    /// Create a new `RoundingIncrement`.
    ///
    /// # Errors
    ///
    /// - If `increment` is less than 1 or bigger than 10**9.
    pub fn try_new(increment: u32) -> TemporalResult<Self> {
        if !(1..=1_000_000_000).contains(&increment) {
            Err(TemporalError::range()
                .with_message("roundingIncrement cannot be less that 1 or bigger than 10**9"))
        } else {
            // SAFETY: The check above guarantees that `increment` is within the
            // specified bounds.
            unsafe { Ok(Self::new_unchecked(increment)) }
        }
    }

    /// Create a new `RoundingIncrement` without checking the validity of the
    /// increment.
    ///
    /// # Safety
    ///
    /// The increment must be equal or bigger than 1, but not bigger than 10**9.
    pub const unsafe fn new_unchecked(increment: u32) -> Self {
        // SAFETY: The caller must ensure the number is bigger than zero.
        unsafe { Self(NonZeroU32::new_unchecked(increment)) }
    }

    // ValidateTemporalRoundingIncrement ( increment, dividend, inclusive )
    // https://tc39.es/proposal-temporal/#sec-validatetemporalroundingincrement
    pub(crate) fn validate(self, dividend: u32, inclusive: bool) -> TemporalResult<()> {
        // 1. If inclusive is true, then
        //     a. Let maximum be dividend.
        // 2. Else,
        //     a. Assert: dividend > 1.
        //     b. Let maximum be dividend - 1.
        let max = if inclusive { dividend } else { dividend - 1 };

        let increment = self.0.get();

        // 3. If increment > maximum, throw a RangeError exception.
        if increment > max {
            return Err(TemporalError::range().with_message("roundingIncrement exceeds maximum."));
        }

        // 4. If dividend modulo increment ≠ 0, then
        if dividend.rem_euclid(increment) != 0 {
            //     a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("dividend is not divisble by roundingIncrement."));
        }

        // 5. Return unused.
        Ok(())
    }
}

// ==== RelativeTo Object ====

pub struct RelativeTo<'a, C: CalendarProtocol, Z: TzProtocol> {
    pub date: Option<&'a Date<C>>,
    pub zdt: Option<&'a ZonedDateTime<C, Z>>,
}

// ==== Options enums and methods ====

/// The relevant unit that should be used for the operation that
/// this option is provided as a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TemporalUnit {
    /// The `Auto` unit
    Auto = 0,
    /// The `Nanosecond` unit
    Nanosecond,
    /// The `Microsecond` unit
    Microsecond,
    /// The `Millisecond` unit
    Millisecond,
    /// The `Second` unit
    Second,
    /// The `Minute` unit
    Minute,
    /// The `Hour` unit
    Hour,
    /// The `Day` unit
    Day,
    /// The `Week` unit
    Week,
    /// The `Month` unit
    Month,
    /// The `Year` unit
    Year,
}

impl TemporalUnit {
    #[inline]
    #[must_use]
    /// Returns the `MaximumRoundingIncrement` for the current `TemporalUnit`.
    pub fn to_maximum_rounding_increment(self) -> Option<u16> {
        use TemporalUnit::{
            Auto, Day, Hour, Microsecond, Millisecond, Minute, Month, Nanosecond, Second, Week,
            Year,
        };
        // 1. If unit is "year", "month", "week", or "day", then
        // a. Return undefined.
        // 2. If unit is "hour", then
        // a. Return 24.
        // 3. If unit is "minute" or "second", then
        // a. Return 60.
        // 4. Assert: unit is one of "millisecond", "microsecond", or "nanosecond".
        // 5. Return 1000.
        match self {
            Year | Month | Week | Day => None,
            Hour => Some(24),
            Minute | Second => Some(60),
            Millisecond | Microsecond | Nanosecond => Some(1000),
            Auto => unreachable!(),
        }
    }

    // TODO: potentiall use a u64
    /// Returns the `Nanosecond amount for any given value.`
    #[must_use]
    pub fn as_nanoseconds(&self) -> Option<f64> {
        use TemporalUnit::{
            Auto, Day, Hour, Microsecond, Millisecond, Minute, Month, Nanosecond, Second, Week,
            Year,
        };
        match self {
            Year | Month | Week | Day | Auto => None,
            Hour => Some(3600e9),
            Minute => Some(60e9),
            Second => Some(1e9),
            Millisecond => Some(1e6),
            Microsecond => Some(1e3),
            Nanosecond => Some(1f64),
        }
    }

    #[must_use]
    pub fn is_calendar_unit(&self) -> bool {
        use TemporalUnit::{Month, Week, Year};
        matches!(self, Year | Month | Week)
    }
}

impl From<usize> for TemporalUnit {
    fn from(value: usize) -> Self {
        match value {
            10 => Self::Year,
            9 => Self::Month,
            8 => Self::Week,
            7 => Self::Day,
            6 => Self::Hour,
            5 => Self::Minute,
            4 => Self::Second,
            3 => Self::Millisecond,
            2 => Self::Microsecond,
            1 => Self::Nanosecond,
            _ => Self::Auto,
        }
    }
}

/// A parsing error for `TemporalUnit`
#[derive(Debug, Clone, Copy)]
pub struct ParseTemporalUnitError;

impl fmt::Display for ParseTemporalUnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid TemporalUnit")
    }
}

impl FromStr for TemporalUnit {
    type Err = ParseTemporalUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "year" | "years" => Ok(Self::Year),
            "month" | "months" => Ok(Self::Month),
            "week" | "weeks" => Ok(Self::Week),
            "day" | "days" => Ok(Self::Day),
            "hour" | "hours" => Ok(Self::Hour),
            "minute" | "minutes" => Ok(Self::Minute),
            "second" | "seconds" => Ok(Self::Second),
            "millisecond" | "milliseconds" => Ok(Self::Millisecond),
            "microsecond" | "microseconds" => Ok(Self::Microsecond),
            "nanosecond" | "nanoseconds" => Ok(Self::Nanosecond),
            _ => Err(ParseTemporalUnitError),
        }
    }
}

impl fmt::Display for TemporalUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => "auto",
            Self::Year => "year",
            Self::Month => "month",
            Self::Week => "week",
            Self::Day => "day",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
            Self::Millisecond => "millsecond",
            Self::Microsecond => "microsecond",
            Self::Nanosecond => "nanosecond",
        }
        .fmt(f)
    }
}

/// `ArithmeticOverflow` can also be used as an
/// assignment overflow and consists of the "constrain"
/// and "reject" options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOverflow {
    /// Constrain option
    Constrain,
    /// Constrain option
    Reject,
}

/// A parsing error for `ArithemeticOverflow`
#[derive(Debug, Clone, Copy)]
pub struct ParseArithmeticOverflowError;

impl fmt::Display for ParseArithmeticOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid overflow value")
    }
}

impl FromStr for ArithmeticOverflow {
    type Err = ParseArithmeticOverflowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constrain" => Ok(Self::Constrain),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseArithmeticOverflowError),
        }
    }
}

impl fmt::Display for ArithmeticOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constrain => "constrain",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}

/// `Duration` overflow options.
#[derive(Debug, Clone, Copy)]
pub enum DurationOverflow {
    /// Constrain option
    Constrain,
    /// Balance option
    Balance,
}

/// A parsing error for `DurationOverflow`.
#[derive(Debug, Clone, Copy)]
pub struct ParseDurationOverflowError;

impl fmt::Display for ParseDurationOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid duration overflow value")
    }
}

impl FromStr for DurationOverflow {
    type Err = ParseDurationOverflowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constrain" => Ok(Self::Constrain),
            "balance" => Ok(Self::Balance),
            _ => Err(ParseDurationOverflowError),
        }
    }
}

impl fmt::Display for DurationOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constrain => "constrain",
            Self::Balance => "balance",
        }
        .fmt(f)
    }
}

/// The disambiguation options for an instant.
#[derive(Debug, Clone, Copy)]
pub enum InstantDisambiguation {
    /// Compatible option
    Compatible,
    /// Earlier option
    Earlier,
    /// Later option
    Later,
    /// Reject option
    Reject,
}

/// A parsing error on `InstantDisambiguation` options.
#[derive(Debug, Clone, Copy)]
pub struct ParseInstantDisambiguationError;

impl fmt::Display for ParseInstantDisambiguationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid instant disambiguation value")
    }
}

impl FromStr for InstantDisambiguation {
    type Err = ParseInstantDisambiguationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compatible" => Ok(Self::Compatible),
            "earlier" => Ok(Self::Earlier),
            "later" => Ok(Self::Later),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseInstantDisambiguationError),
        }
    }
}

impl fmt::Display for InstantDisambiguation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compatible => "compatible",
            Self::Earlier => "earlier",
            Self::Later => "later",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}

/// Offset disambiguation options.
#[derive(Debug, Clone, Copy)]
pub enum OffsetDisambiguation {
    /// Use option
    Use,
    /// Prefer option
    Prefer,
    /// Ignore option
    Ignore,
    /// Reject option
    Reject,
}

/// A parsing error for `OffsetDisambiguation` parsing.
#[derive(Debug, Clone, Copy)]
pub struct ParseOffsetDisambiguationError;

impl fmt::Display for ParseOffsetDisambiguationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid offset disambiguation value")
    }
}

impl FromStr for OffsetDisambiguation {
    type Err = ParseOffsetDisambiguationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "use" => Ok(Self::Use),
            "prefer" => Ok(Self::Prefer),
            "ignore" => Ok(Self::Ignore),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseOffsetDisambiguationError),
        }
    }
}

impl fmt::Display for OffsetDisambiguation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Use => "use",
            Self::Prefer => "prefer",
            Self::Ignore => "ignore",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}

// TODO: Figure out what to do with intl's RoundingMode

/// Declares the specified `RoundingMode` for the operation.
#[derive(Debug, Copy, Clone, Default)]
pub enum TemporalRoundingMode {
    /// Ceil RoundingMode
    Ceil,
    /// Floor RoundingMode
    Floor,
    /// Expand RoundingMode
    Expand,
    /// Truncate RoundingMode
    Trunc,
    /// HalfCeil RoundingMode
    HalfCeil,
    /// HalfFloor RoundingMode
    HalfFloor,
    /// HalfExpand RoundingMode - Default
    #[default]
    HalfExpand,
    /// HalfTruncate RoundingMode
    HalfTrunc,
    /// HalfEven RoundingMode
    HalfEven,
}

/// The `UnsignedRoundingMode`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalUnsignedRoundingMode {
    /// `Infinity` `RoundingMode`
    Infinity,
    /// `Zero` `RoundingMode`
    Zero,
    /// `HalfInfinity` `RoundingMode`
    HalfInfinity,
    /// `HalfZero` `RoundingMode`
    HalfZero,
    /// `HalfEven` `RoundingMode`
    HalfEven,
}

impl TemporalRoundingMode {
    #[inline]
    #[must_use]
    /// Negates the current `RoundingMode`.
    pub const fn negate(self) -> Self {
        use TemporalRoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil => Self::Floor,
            Floor => Self::Ceil,
            HalfCeil => Self::HalfFloor,
            HalfFloor => Self::HalfCeil,
            Trunc => Self::Trunc,
            Expand => Self::Expand,
            HalfTrunc => Self::HalfTrunc,
            HalfExpand => Self::HalfExpand,
            HalfEven => Self::HalfEven,
        }
    }

    #[inline]
    #[must_use]
    /// Returns the `UnsignedRoundingMode`
    pub const fn get_unsigned_round_mode(self, is_negative: bool) -> TemporalUnsignedRoundingMode {
        use TemporalRoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil if !is_negative => TemporalUnsignedRoundingMode::Infinity,
            Ceil => TemporalUnsignedRoundingMode::Zero,
            Floor if !is_negative => TemporalUnsignedRoundingMode::Zero,
            Floor | Trunc | Expand => TemporalUnsignedRoundingMode::Infinity,
            HalfCeil if !is_negative => TemporalUnsignedRoundingMode::HalfInfinity,
            HalfCeil | HalfTrunc => TemporalUnsignedRoundingMode::HalfZero,
            HalfFloor if !is_negative => TemporalUnsignedRoundingMode::HalfZero,
            HalfFloor | HalfExpand => TemporalUnsignedRoundingMode::HalfInfinity,
            HalfEven => TemporalUnsignedRoundingMode::HalfEven,
        }
    }
}

impl FromStr for TemporalRoundingMode {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ceil" => Ok(Self::Ceil),
            "floor" => Ok(Self::Floor),
            "expand" => Ok(Self::Expand),
            "trunc" => Ok(Self::Trunc),
            "halfCeil" => Ok(Self::HalfCeil),
            "halfFloor" => Ok(Self::HalfFloor),
            "halfExpand" => Ok(Self::HalfExpand),
            "halfTrunc" => Ok(Self::HalfTrunc),
            "halfEven" => Ok(Self::HalfEven),
            _ => Err(TemporalError::range().with_message("RoundingMode not an accepted value.")),
        }
    }
}

impl fmt::Display for TemporalRoundingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ceil => "ceil",
            Self::Floor => "floor",
            Self::Expand => "expand",
            Self::Trunc => "trunc",
            Self::HalfCeil => "halfCeil",
            Self::HalfFloor => "halfFloor",
            Self::HalfExpand => "halfExpand",
            Self::HalfTrunc => "halfTrunc",
            Self::HalfEven => "halfEven",
        }
        .fmt(f)
    }
}
