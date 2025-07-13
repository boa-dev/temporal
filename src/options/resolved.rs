//! A module for all resolved option types

use core::{fmt, str::FromStr};
use alloc::string::ToString;

use crate::{
    options::{RoundingIncrement, RoundingMode, Unit}, TemporalError, TemporalResult
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
    pub negated_rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ResolvedPlainDateLargestUnit(ResolvedUnit);

impl ResolvedPlainDateLargestUnit {
    pub fn try_from_units(unit: DateUnitOrAuto, resolved_smallest_unit: ResolvedPlainDateSmallestUnit) -> TemporalResult<Self> {
        // NOTE: Types enforce no auto invariant.
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        let default_largest = Unit::larger(Unit::Day, resolved_smallest_unit.0.0).expect("no auto");

        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        let largest_unit = if unit.0 == Unit::Auto {
            ResolvedUnit::try_from_unit(default_largest)?
        } else {
            ResolvedUnit::try_from_unit(unit.0)?
        };
        
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if ResolvedUnit::larger(largest_unit, resolved_smallest_unit.0) != largest_unit {
            return Err(TemporalError::range())
        }
        Ok(Self(largest_unit))
    }
}

const DATE_UNITS:[Unit; 4] = [Unit::Year, Unit::Month, Unit::Week, Unit::Day];

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ResolvedPlainDateSmallestUnit(ResolvedUnit);

impl ResolvedPlainDateSmallestUnit {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if !DATE_UNITS.contains(&unit) {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(ResolvedUnit(unit)))
    }

    pub fn to_maximum_rounding_increment(&self) -> Option<u32> {
        self.0.0.to_maximum_rounding_increment()
    }
}

#[derive(Debug, Clone, Copy)]
// Required for Plain Date largest unit resolution
#[repr(transparent)]
pub struct DateUnitOrAuto(Unit);

impl DateUnitOrAuto {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if !DATE_UNITS.contains(&unit) && unit != Unit::Auto {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(unit))
    }
}

impl FromStr for DateUnitOrAuto {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit = Unit::from_str(s)
            .map_err(|e| TemporalError::range().with_message(e.to_string()))?;
        Self::try_from_unit(unit)
    }
}

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
    pub negated_rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedPlainDateTimeLargestUnit(ResolvedUnit);

impl ResolvedPlainDateTimeLargestUnit {
    pub fn try_from_units(unit: Unit, resolved_smallest_unit: ResolvedPlainDateTimeSmallestUnit) -> TemporalResult<Self> {
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        let default_largest = ResolvedUnit::larger(ResolvedUnit(Unit::Day), resolved_smallest_unit.0);

        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        let largest_unit = if unit == Unit::Auto {
            default_largest
        } else {
            ResolvedUnit::try_from_unit(unit)?
        };
        
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if ResolvedUnit::larger(largest_unit, resolved_smallest_unit.0) != largest_unit {
            return Err(TemporalError::range())
        }
        Ok(Self(largest_unit))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedPlainDateTimeSmallestUnit(ResolvedUnit);

impl ResolvedPlainDateTimeSmallestUnit {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if unit != Unit::Auto {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(ResolvedUnit(unit)))
    }
}

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
    pub negated_rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedZonedDateTimeLargestUnit(ResolvedUnit);

impl ResolvedZonedDateTimeLargestUnit {
    pub fn try_from_units(unit: Unit, resolved_smallest_unit: ResolvedPlainDateTimeSmallestUnit) -> TemporalResult<Self> {
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        let default_largest = ResolvedUnit::larger(ResolvedUnit(Unit::Hour), resolved_smallest_unit.0);

        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        let largest_unit = if unit == Unit::Auto {
            default_largest
        } else {
            ResolvedUnit::try_from_unit(unit)?
        };
        
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if ResolvedUnit::larger(largest_unit, resolved_smallest_unit.0) != largest_unit {
            return Err(TemporalError::range())
        }
        Ok(Self(largest_unit))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedZonedDateTimeSmallestUnit(ResolvedUnit);

impl ResolvedZonedDateTimeSmallestUnit {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if unit != Unit::Auto {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(ResolvedUnit(unit)))
    }
}

// ==== PlainTime variation ====

pub struct ResolvedTimeRoundOptions {
    pub largest_unit: ResolvedTimeLargestUnit, // Potentially need different types or methods
    pub smallest_unit: ResolvedTimeSmallestUnit, // Potentially need different types or methods
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

const TIME_UNITS: [Unit; 6] = [
    Unit::Hour,
    Unit::Minute,
    Unit::Second,
    Unit::Millisecond,
    Unit::Microsecond,
    Unit::Nanosecond,
];

#[derive(Debug, Clone, Copy)]
pub struct ResolvedTimeLargestUnit(ResolvedUnit);

impl ResolvedTimeLargestUnit {
    pub fn try_from_units(unit: Unit, resolved_smallest_unit: ResolvedPlainDateTimeSmallestUnit) -> TemporalResult<Self> {
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        let default_largest = ResolvedUnit::larger(ResolvedUnit(Unit::Hour), resolved_smallest_unit.0);

        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        let largest_unit = if unit == Unit::Auto {
            default_largest
        } else {
            ResolvedUnit::try_from_unit(unit)?
        };
        
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if ResolvedUnit::larger(largest_unit, resolved_smallest_unit.0) != largest_unit {
            return Err(TemporalError::range())
        }
        Ok(Self(largest_unit))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedTimeSmallestUnit(ResolvedUnit);

impl ResolvedTimeSmallestUnit {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if !TIME_UNITS.contains(&unit) {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(ResolvedUnit(unit)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeUnitOrAuto(Unit);

impl TimeUnitOrAuto {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if !TIME_UNITS.contains(&unit) && unit != Unit::Auto {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(unit))
    }
}

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
    pub negated_rounding_mode: NegatedRoundingMode,
    pub increment: RoundingIncrement,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedYearMonthLargestUnit(ResolvedUnit);

impl ResolvedYearMonthLargestUnit {
    pub fn try_from_units(unit: YearMonthUnitOrAuto, resolved_smallest_unit: ResolvedYearMonthSmallestUnit) -> TemporalResult<Self> {
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        let default_largest = ResolvedUnit::larger(ResolvedUnit(Unit::Hour), resolved_smallest_unit.0);

        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        let largest_unit = if unit.0 == Unit::Auto {
            default_largest
        } else {
            ResolvedUnit::try_from_unit(unit.0)?
        };
        
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if ResolvedUnit::larger(largest_unit, resolved_smallest_unit.0) != largest_unit {
            return Err(TemporalError::range())
        }
        Ok(Self(largest_unit))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedYearMonthSmallestUnit(ResolvedUnit);

#[derive(Debug, Clone, Copy)]
pub struct YearMonthUnitOrAuto(Unit);

impl YearMonthUnitOrAuto {
    pub fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if ![Unit::Year, Unit::Month].contains(&unit) && unit != Unit::Auto {
            // TODO: RangeError message.
            return Err(TemporalError::range())
        }
        Ok(Self(unit))
    }
}

// ==== Extra options ====

pub struct NegatedRoundingMode(RoundingMode);

impl From<RoundingMode> for NegatedRoundingMode {
    fn from(value: RoundingMode) -> Self {
        NegatedRoundingMode(value.negate())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ResolvedUnit(Unit);

impl ResolvedUnit {
    fn try_from_unit(unit: Unit) -> TemporalResult<Self> {
        if unit == Unit::Auto {
            return Err(TemporalError::range())
        }
        Ok(Self(unit))
    }

    fn larger(u1: ResolvedUnit, u2: ResolvedUnit) -> ResolvedUnit {
        Self(Unit::larger(u1.0, u2.0).expect("no auto"))

    }
}

impl FromStr for ResolvedUnit {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit = Unit::from_str(s).map_err(|_| TemporalError::range())?;
        Self::try_from_unit(unit)
    }
}

impl fmt::Display for ResolvedUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}


#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::options::{DateUnitOrAuto, NegatedRoundingMode, ResolvedPlainDateLargestUnit, ResolvedPlainDateSinceDifferenceSettings, ResolvedPlainDateSmallestUnit, ResolvedPlainDateUntilDifferenceSettings, RoundingIncrement, RoundingMode, Unit};

    #[test]
    fn impl_plain_date_get_difference_settings() {
        struct JsOptions {
            smallest_unit: &'static str,
            largest_unit: &'static str,
            increment: u32,
            rounding_mode: &'static str,
        }

        let js_options = JsOptions {
            smallest_unit: "day",
            largest_unit: "auto",
            increment: 1,
            rounding_mode: "floor",
        };

        // 1. NOTE: The following steps read options and perform independent validation in alphabetical order.
        // 2. Let largestUnit be ? GetTemporalUnitValuedOption(options, "largestUnit", unitGroup, auto).
        // 3. If disallowedUnits contains largestUnit, throw a RangeError exception.
        let unit = Unit::from_str(js_options.largest_unit).unwrap();
        let largest_unit = DateUnitOrAuto::try_from_unit(unit).unwrap();
        // 4. Let roundingIncrement be ? GetRoundingIncrementOption(options).
        let increment = RoundingIncrement::try_new(js_options.increment).unwrap();
        // 5. Let roundingMode be ? GetRoundingModeOption(options, trunc).
        let rounding_mode = RoundingMode::from_str(js_options.rounding_mode).unwrap();
        // 6. If operation is since, then
        // a. Set roundingMode to NegateRoundingMode(roundingMode).
        let negated_rounding_mode = NegatedRoundingMode::from(rounding_mode);
        // 7. Let smallestUnit be ? GetTemporalUnitValuedOption(options, "smallestUnit", unitGroup, fallbackSmallestUnit).
        // 8. If disallowedUnits contains smallestUnit, throw a RangeError exception.
        let unit = Unit::from_str(js_options.smallest_unit).unwrap();
        let smallest_unit = ResolvedPlainDateSmallestUnit::try_from_unit(unit).unwrap();
        // 9. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
        // 10. If largestUnit is auto, set largestUnit to defaultLargestUnit.
        // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        let largest_unit = ResolvedPlainDateLargestUnit::try_from_units(largest_unit, smallest_unit).unwrap();
        // 12. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        let maximum = smallest_unit.to_maximum_rounding_increment();
        // 13. If maximum is not unset, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if let Some(max) = maximum {
            increment.validate(max.into(), false).unwrap();
        }
        // 14. Return the Record { [[SmallestUnit]]: smallestUnit, [[LargestUnit]]: largestUnit, [[RoundingMode]]: roundingMode, [[RoundingIncrement]]: roundingIncrement,  }.
        let _until_options = ResolvedPlainDateUntilDifferenceSettings {
            largest_unit,
            smallest_unit,
            rounding_mode,
            increment,
        };

        let _since_options = ResolvedPlainDateSinceDifferenceSettings {
            largest_unit,
            smallest_unit,
            negated_rounding_mode,
            increment,
        };
    }
}