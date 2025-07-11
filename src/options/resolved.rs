//! A module for all resolved option types

use crate::options::{RoundingIncrement, RoundingMode};

pub struct ResolvedDurationRoundOptions {
    pub largest_unit: ResolvedDurationLargestUnit,
    pub smallest_unit: ResolvedDurationSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

pub struct ResolvedDurationLargestUnit(ResolvedUnit);

pub struct ResolvedDurationSmallestUnit(ResolvedUnit);

// ==== PlainDate variation ====

pub struct ResolvedPlainDateRoundOptions {
    pub largest_unit: ResolvedDurationLargestUnit,
    pub smallest_unit: ResolvedDurationSmallestUnit,
    pub rounding_mode: RoundingMode,
    pub increment: RoundingIncrement,
}

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
