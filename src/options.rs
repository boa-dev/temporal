//! Native implementation of the `Temporal` options.
//!
//! Temporal has various instances where user's can define options for how an
//! operation may be completed.

use crate::{Sign, TemporalError, TemporalResult, MS_PER_DAY, NS_PER_DAY};
use core::ops::Add;
use core::{fmt, str::FromStr};

mod increment;
mod relative_to;

pub use increment::RoundingIncrement;
pub use relative_to::RelativeTo;

// ==== RoundingOptions / DifferenceSettings ====

#[derive(Debug, Clone, Copy)]
pub(crate) enum DifferenceOperation {
    Until,
    Since,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct DifferenceSettings {
    pub largest_unit: Option<TemporalUnit>,
    pub smallest_unit: Option<TemporalUnit>,
    pub rounding_mode: Option<TemporalRoundingMode>,
    pub increment: Option<RoundingIncrement>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct RoundingOptions {
    pub largest_unit: Option<TemporalUnit>,
    pub smallest_unit: Option<TemporalUnit>,
    pub rounding_mode: Option<TemporalRoundingMode>,
    pub increment: Option<RoundingIncrement>,
}

// Note: Specification does not clearly state a default, but
// having both largest and smallest unit None would auto throw.

impl Default for RoundingOptions {
    fn default() -> Self {
        Self {
            largest_unit: Some(TemporalUnit::Auto),
            smallest_unit: None,
            rounding_mode: None,
            increment: None,
        }
    }
}

/// Internal options object that represents the resolved rounding options.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedRoundingOptions {
    pub(crate) largest_unit: TemporalUnit,
    pub(crate) smallest_unit: TemporalUnit,
    pub(crate) increment: RoundingIncrement,
    pub(crate) rounding_mode: TemporalRoundingMode,
}

impl ResolvedRoundingOptions {
    pub(crate) fn from_diff_settings(
        options: DifferenceSettings,
        operation: DifferenceOperation,
        fallback_largest: TemporalUnit,
        fallback_smallest: TemporalUnit,
    ) -> TemporalResult<(Sign, Self)> {
        // 4. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 5. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, DATE, « », "day", "day").
        let increment = options.increment.unwrap_or_default();
        let (sign, rounding_mode) = match operation {
            DifferenceOperation::Since => {
                let mode = options
                    .rounding_mode
                    .unwrap_or(TemporalRoundingMode::Trunc)
                    .negate();
                (Sign::Negative, mode)
            }
            DifferenceOperation::Until => (
                Sign::Positive,
                options.rounding_mode.unwrap_or(TemporalRoundingMode::Trunc),
            ),
        };
        let smallest_unit = options.smallest_unit.unwrap_or(fallback_smallest);
        // Use the defaultlargestunit which is max smallestlargestdefault and smallestunit
        let largest_unit = options
            .largest_unit
            .unwrap_or(smallest_unit.max(fallback_largest));

        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        // 12. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        // 13. If maximum is not unset, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if largest_unit.max(smallest_unit) != largest_unit {
            return Err(TemporalError::range().with_message(
                "largestUnit when rounding Duration was not the largest provided unit",
            ));
        }

        let maximum = smallest_unit.to_maximum_rounding_increment();
        if let Some(max) = maximum {
            increment.validate(max.into(), false)?;
        }

        let resolved = ResolvedRoundingOptions {
            largest_unit,
            smallest_unit,
            increment,
            rounding_mode,
        };

        Ok((sign, resolved))
    }

    pub(crate) fn from_duration_options(
        options: RoundingOptions,
        existing_largest: TemporalUnit,
    ) -> TemporalResult<Self> {
        // 22. If smallestUnitPresent is false and largestUnitPresent is false, then
        if options.largest_unit.is_none() && options.smallest_unit.is_none() {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("smallestUnit and largestUnit cannot both be None."));
        }

        // 14. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let increment = options.increment.unwrap_or_default();
        // 15. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode = options.rounding_mode.unwrap_or_default();
        // 16. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", DATETIME, undefined).
        // 17. If smallestUnit is undefined, then
        // a. Set smallestUnitPresent to false.
        // b. Set smallestUnit to "nanosecond".
        // 18. Let existingLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]],
        // duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]],
        // duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        // duration.[[Microseconds]]).
        // 19. Let defaultLargestUnit be LargerOfTwoTemporalUnits(existingLargestUnit, smallestUnit).
        // 20. If largestUnit is undefined, then
        // a. Set largestUnitPresent to false.
        // b. Set largestUnit to defaultLargestUnit.
        // 21. Else if largestUnit is "auto", then
        // a. Set largestUnit to defaultLargestUnit.
        // 23. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        // 24. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        // 25. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        let smallest_unit = options.smallest_unit.unwrap_or(TemporalUnit::Nanosecond);

        let default_largest = existing_largest.max(smallest_unit);

        let largest_unit = match options.largest_unit {
            Some(TemporalUnit::Auto) | None => default_largest,
            Some(unit) => unit,
        };

        if largest_unit.max(smallest_unit) != largest_unit {
            return Err(TemporalError::range().with_message(
                "largestUnit when rounding Duration was not the largest provided unit",
            ));
        }

        let maximum = smallest_unit.to_maximum_rounding_increment();
        // 25. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if let Some(max) = maximum {
            increment.validate(max.into(), false)?;
        }

        Ok(Self {
            largest_unit,
            smallest_unit,
            increment,
            rounding_mode,
        })
    }

    // NOTE: Should the GetTemporalUnitValuedOption check be integrated into these validations.
    pub(crate) fn from_dt_options(options: RoundingOptions) -> TemporalResult<Self> {
        let increment = options.increment.unwrap_or_default();
        let rounding_mode = options.rounding_mode.unwrap_or_default();
        let smallest_unit = options.smallest_unit.unwrap_or(TemporalUnit::Day);
        let (maximum, inclusive) = if smallest_unit == TemporalUnit::Day {
            (1, true)
        } else {
            let maximum = smallest_unit
                .to_maximum_rounding_increment()
                .ok_or(TemporalError::range().with_message("smallestUnit must be a time unit."))?;
            (maximum, false)
        };

        increment.validate(maximum.into(), inclusive)?;

        Ok(Self {
            largest_unit: TemporalUnit::Auto,
            smallest_unit,
            increment,
            rounding_mode,
        })
    }

    pub(crate) fn from_instant_options(options: RoundingOptions) -> TemporalResult<Self> {
        let increment = options.increment.unwrap_or_default();
        let rounding_mode = options.rounding_mode.unwrap_or_default();
        let Some(smallest_unit) = options.smallest_unit else {
            return Err(TemporalError::range()
                .with_message("smallestUnit is required for an Instant.round operation."));
        };
        let maximum = match smallest_unit {
            TemporalUnit::Hour => 24u64,
            TemporalUnit::Minute => 24 * 60,
            TemporalUnit::Second => 24 * 3600,
            TemporalUnit::Millisecond => MS_PER_DAY as u64,
            TemporalUnit::Microsecond => MS_PER_DAY as u64 * 1000,
            TemporalUnit::Nanosecond => NS_PER_DAY,
            _ => return Err(TemporalError::range().with_message("Invalid roundTo unit provided.")),
        };

        increment.validate(maximum, true)?;

        Ok(Self {
            largest_unit: TemporalUnit::Auto,
            smallest_unit,
            increment,
            rounding_mode,
        })
    }

    pub(crate) fn is_noop(&self) -> bool {
        self.smallest_unit == TemporalUnit::Nanosecond && self.increment == RoundingIncrement::ONE
    }
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
    pub fn to_maximum_rounding_increment(self) -> Option<u32> {
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
        let max = match self {
            Year | Month | Week | Day => return None,
            Hour => 24,
            Minute | Second => 60,
            Millisecond | Microsecond | Nanosecond => 1000,
            Auto => unreachable!(),
        };

        Some(max)
    }

    // TODO: potentiall use a u64
    /// Returns the `Nanosecond amount for any given value.`
    #[must_use]
    pub fn as_nanoseconds(&self) -> Option<u64> {
        use TemporalUnit::{
            Auto, Day, Hour, Microsecond, Millisecond, Minute, Month, Nanosecond, Second, Week,
            Year,
        };
        match self {
            Year | Month | Week | Auto => None,
            Day => Some(NS_PER_DAY),
            Hour => Some(3_600_000_000_000),
            Minute => Some(60_000_000_000),
            Second => Some(1_000_000_000),
            Millisecond => Some(1_000_000),
            Microsecond => Some(1_000),
            Nanosecond => Some(1),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_calendar_unit(&self) -> bool {
        use TemporalUnit::{Month, Week, Year};
        matches!(self, Year | Month | Week)
    }

    #[inline]
    #[must_use]
    pub fn is_time_unit(&self) -> bool {
        use TemporalUnit::{Hour, Microsecond, Millisecond, Minute, Nanosecond, Second};
        matches!(
            self,
            Hour | Minute | Second | Millisecond | Microsecond | Nanosecond
        )
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

impl Add<usize> for TemporalUnit {
    type Output = TemporalUnit;

    fn add(self, rhs: usize) -> Self::Output {
        TemporalUnit::from(self as usize + rhs)
    }
}

/// A parsing error for `TemporalUnit`
#[derive(Debug, Clone, Copy)]
pub struct ParseTemporalUnitError;

impl fmt::Display for ParseTemporalUnitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOverflow {
    /// Constrain option
    #[default]
    Constrain,
    /// Constrain option
    Reject,
}

/// A parsing error for `ArithemeticOverflow`
#[derive(Debug, Clone, Copy)]
pub struct ParseArithmeticOverflowError;

impl fmt::Display for ParseArithmeticOverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Constrain => "constrain",
            Self::Balance => "balance",
        }
        .fmt(f)
    }
}

/// The disambiguation options for an instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disambiguation {
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
pub struct ParseDisambiguationError;

impl fmt::Display for ParseDisambiguationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("provided string was not a valid instant disambiguation value")
    }
}

impl FromStr for Disambiguation {
    type Err = ParseDisambiguationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compatible" => Ok(Self::Compatible),
            "earlier" => Ok(Self::Earlier),
            "later" => Ok(Self::Later),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseDisambiguationError),
        }
    }
}

impl fmt::Display for Disambiguation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    pub const fn get_unsigned_round_mode(self, is_positive: bool) -> TemporalUnsignedRoundingMode {
        use TemporalRoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil if is_positive => TemporalUnsignedRoundingMode::Infinity,
            Ceil => TemporalUnsignedRoundingMode::Zero,
            Floor if is_positive => TemporalUnsignedRoundingMode::Zero,
            Floor | Trunc | Expand => TemporalUnsignedRoundingMode::Infinity,
            HalfCeil if is_positive => TemporalUnsignedRoundingMode::HalfInfinity,
            HalfCeil | HalfTrunc => TemporalUnsignedRoundingMode::HalfZero,
            HalfFloor if is_positive => TemporalUnsignedRoundingMode::HalfZero,
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

/// values for `CalendarName`, whether to show the calendar in toString() methods
/// <https://tc39.es/proposal-temporal/#sec-temporal-gettemporalshowcalendarnameoption>
#[derive(Debug, Clone, Copy)]
pub enum CalendarName {
    /// `Auto` option
    Auto,
    /// `Always` option
    Always,
    /// `Never` option
    Never,
    // `Critical` option
    Critical,
}

impl fmt::Display for CalendarName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CalendarName::Auto => "auto",
            CalendarName::Always => "always",
            CalendarName::Never => "never",
            CalendarName::Critical => "critical",
        }
        .fmt(f)
    }
}

impl FromStr for CalendarName {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            "critical" => Ok(Self::Critical),
            _ => Err(TemporalError::range().with_message("Invalid CalendarName provided.")),
        }
    }
}
