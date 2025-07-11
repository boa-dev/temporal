//! A module for all resolved option types

use core::{fmt, str::FromStr};

use crate::{
    options::{RoundingIncrement, RoundingMode, Unit},
    TemporalResult,
};

pub struct ResolvedDurationRoundOptions {
    pub largest_unit: ResolvedUnit,
    pub smallest_unit: ResolvedUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

// ==== PlainDate variation ====

pub struct ResolvedPlainDateUntilDifferenceSettings {
    pub largest_unit: ResolvedPlainDateLargestUnit,
    pub smallest_unit: ResolvedPlainDateSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedPlainDateSinceDifferenceSettings {
    pub largest_unit: ResolvedPlainDateLargestUnit,
    pub smallest_unit: ResolvedPlainDateSmallestUnit,
    pub rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedPlainDateLargestUnit(ResolvedUnit);

pub struct ResolvedPlainDateSmallestUnit(ResolvedUnit);

// ==== PlainDateTime variation ====

pub struct ResolvedPlainDateTimeRoundOptions {
    pub largest_unit: ResolvedPlainDateLargestUnit,
    pub smallest_unit: ResolvedPlainDateTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedPlainDateTimeUntilDifferenceSettings {
    pub largest_unit: ResolvedPlainDateTimeLargestUnit,
    pub smallest_unit: ResolvedPlainDateTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedPlainDateTimeSinceDifferenceSettings {
    pub largest_unit: ResolvedPlainDateLargestUnit,
    pub smallest_unit: ResolvedPlainDateLargestUnit,
    pub rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedPlainDateTimeLargestUnit(ResolvedUnit);

pub struct ResolvedPlainDateTimeSmallestUnit(ResolvedUnit);

// ==== ZonedDateTime variation ====

pub struct ResolvedZonedDateTimeRoundOptions {
    pub largest_unit: ResolvedPlainDateTimeLargestUnit,
    pub smallest_unit: ResolvedPlainDateTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedZonedDateTimeUntilDifferenceSettings {
    pub largest_unit: ResolvedPlainDateTimeLargestUnit,
    pub smallest_unit: ResolvedPlainDateTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedZonedDateTimeSinceDifferenceSettings {
    pub largest_unit: ResolvedPlainDateTimeLargestUnit,
    pub smallest_unit: ResolvedPlainDateTimeSmallestUnit,
    pub rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedZonedDateTimeLargestUnit(ResolvedUnit);

pub struct ResolvedZonedDateTimeSmallestUnit(ResolvedUnit);

// ==== PlainTime variation ====

pub struct ResolvedTimeRoundOptions {
    pub largest_unit: ResolvedTimeLargestUnit,
    pub smallest_unit: ResolvedTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedTimeUntilDifferenceSettings {
    pub largest_unit: ResolvedTimeLargestUnit,
    pub smallest_unit: ResolvedTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedTimeSinceDifferenceSettings {
    pub largest_unit: ResolvedTimeLargestUnit,
    pub smallest_unit: ResolvedTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedTimeLargestUnit(ResolvedUnit);

pub struct ResolvedTimeSmallestUnit(ResolvedUnit);

// ==== YearMonth variation ====

pub struct ResolvedYearMonthUntilDifferenceSettings {
    pub largest_unit: ResolvedZonedDateTimeLargestUnit,
    pub smallest_unit: ResolvedZonedDateTimeSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedYearMonthSinceDifferenceSettings {
    pub largest_unit: ResolvedZonedDateTimeLargestUnit,
    pub smallest_unit: ResolvedZonedDateTimeSmallestUnit,
    pub rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedYearMonthLargestUnit(ResolvedUnit);

pub struct ResolvedYearMonthSmallestUnit(ResolvedUnit);

pub struct NegatedRoundingMode(RoundingMode);

impl From<RoundingMode> for NegatedRoundingMode {
    fn from(value: RoundingMode) -> Self {
        NegatedRoundingMode(value.negate())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedUnit {
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
/// A parsing error for `Unit`
#[derive(Debug, Clone, Copy)]
pub struct ParseResolvedUnitError;

impl fmt::Display for ParseResolvedUnitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("provided string was not a valid Unit")
    }
}

impl FromStr for ResolvedUnit {
    type Err = ParseResolvedUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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
            _ => Err(ParseResolvedUnitError),
        }
    }
}

impl fmt::Display for ResolvedUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
