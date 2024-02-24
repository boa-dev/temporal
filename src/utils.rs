//! Utility date and time equations for Temporal

use std::{cmp::Ordering, ops::Neg};

use crate::{
    options::{TemporalRoundingMode, TemporalUnsignedRoundingMode},
    TemporalError, TemporalResult, MS_PER_DAY,
};

// NOTE: Review the below for optimizations and add ALOT of tests.

/// Converts and validates an `Option<f64>` rounding increment value into a valid increment result.
pub(crate) fn to_rounding_increment(increment: Option<f64>) -> TemporalResult<u64> {
    let inc = increment.unwrap_or(1.0);

    if !inc.is_finite() {
        return Err(TemporalError::range().with_message("roundingIncrement must be finite."));
    }

    let integer = inc.trunc();

    if !(1.0..=1_000_000_000f64).contains(&integer) {
        return Err(
            TemporalError::range().with_message("roundingIncrement is not within a valid range.")
        );
    }

    Ok(integer as u64)
}

/// Applies the unsigned rounding mode.
fn apply_unsigned_rounding_mode(
    quotient: f64,
    floor: f64,
    ceil: f64,
    unsigned_rounding_mode: TemporalUnsignedRoundingMode,
) -> u64 {
    // 1. If x is equal to r1, return r1.
    if quotient == floor {
        return floor as u64;
    }
    // 2. Assert: r1 < x < r2.
    // 3. Assert: unsignedRoundingMode is not undefined.

    // 4. If unsignedRoundingMode is zero, return r1.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Zero {
        return floor as u64;
    };
    // 5. If unsignedRoundingMode is infinity, return r2.
    if unsigned_rounding_mode == TemporalUnsignedRoundingMode::Infinity {
        return ceil as u64;
    };

    // 6. Let d1 be x – r1.
    // 7. Let d2 be r2 – x.
    let d1 = quotient - floor;
    let d2 = ceil - quotient;
    // 8. If d1 < d2, return r1.
    // 9. If d2 < d1, return r2.
    match d1.partial_cmp(&d2) {
        Some(Ordering::Less) => floor as u64,
        Some(Ordering::Greater) => ceil as u64,
        Some(Ordering::Equal) => {
            // 10. Assert: d1 is equal to d2.
            // 11. If unsignedRoundingMode is half-zero, return r1.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfZero {
                return floor as u64;
            };
            // 12. If unsignedRoundingMode is half-infinity, return r2.
            if unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfInfinity {
                return ceil as u64;
            };
            // 13. Assert: unsignedRoundingMode is half-even.
            assert!(unsigned_rounding_mode == TemporalUnsignedRoundingMode::HalfEven);
            // 14. Let cardinality be (r1 / (r2 – r1)) modulo 2.
            let cardinality = (floor / (ceil - floor)) % 2.0;
            // 15. If cardinality is 0, return r1.
            if cardinality == 0.0 {
                return floor as u64;
            }
            // 16. Return r2.
            ceil as u64
        }
        None => unreachable!(),
    }
}

// TODO: Use `div_ceil` and `div_floor` once stable.
// Tracking issue: https://github.com/rust-lang/rust/issues/88581
/// 13.28 `RoundNumberToIncrement ( x, increment, roundingMode )`
pub(crate) fn round_number_to_increment(
    x: f64,
    increment: f64,
    rounding_mode: TemporalRoundingMode,
) -> i64 {
    // 1. Let quotient be x / increment.
    let quotient = x / increment;
    // 2. If quotient < 0, then
    let (is_negative, quotient) = if quotient < 0.0 {
        // a. Let isNegative be true.
        // b. Set quotient to -quotient.
        (true, quotient.abs())
    // 3. Else,
    } else {
        // a. Let isNegative be false.
        (false, quotient)
    };

    // 4. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, isNegative).
    let unsigned_rounding_mode = rounding_mode.get_unsigned_round_mode(is_negative);

    // 5. Let r1 be the largest integer such that r1 ≤ quotient.
    let floor = quotient.floor();
    // 6. Let r2 be the smallest integer such that r2 > quotient.
    let ceil = quotient.ceil();

    // 7. Let rounded be ApplyUnsignedRoundingMode(quotient, r1, r2, unsignedRoundingMode).
    let rounded = apply_unsigned_rounding_mode(quotient, floor, ceil, unsigned_rounding_mode);

    // 8. If isNegative is true, set rounded to -rounded.
    let rounded = if is_negative {
        (rounded as i64).neg()
    } else {
        rounded as i64
    };

    // 9. Return rounded × increment.
    rounded * (increment as i64)
}

/// Rounds provided number assuming that the increment is greater than 0.
pub(crate) fn round_number_to_increment_as_if_positive(
    x: f64,
    increment: f64,
    rounding_mode: TemporalRoundingMode,
) -> u64 {
    // 1. Let quotient be x / increment.
    let quotient = x / increment;
    // 2. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, false).
    let unsigned_rounding_mode = rounding_mode.get_unsigned_round_mode(false);
    // 3. Let r1 be the largest integer such that r1 ≤ quotient.
    let r1 = quotient.floor();
    // 4. Let r2 be the smallest integer such that r2 > quotient.
    let r2 = quotient.ceil();
    // 5. Let rounded be ApplyUnsignedRoundingMode(quotient, r1, r2, unsignedRoundingMode).
    let rounded = apply_unsigned_rounding_mode(quotient, r1, r2, unsigned_rounding_mode);
    // 6. Return rounded × increment.
    rounded * (increment as u64)
}

pub(crate) fn validate_temporal_rounding_increment(
    increment: u64,
    dividend: u64,
    inclusive: bool,
) -> TemporalResult<()> {
    let max = if inclusive { dividend } else { dividend - 1 };

    if increment > max {
        return Err(TemporalError::range().with_message("roundingIncrement exceeds maximum."));
    }

    if dividend.rem_euclid(increment) != 0 {
        return Err(
            TemporalError::range().with_message("dividend is not divisble by roundingIncrement.")
        );
    }

    Ok(())
}

// ==== Begin Date Equations ====

pub(crate) const MS_PER_HOUR: f64 = 3_600_000f64;
pub(crate) const MS_PER_MINUTE: f64 = 60_000f64;

/// `EpochDaysToEpochMS`
///
/// Functionally the same as Date's abstract operation `MakeDate`
pub(crate) fn epoch_days_to_epoch_ms(day: i32, time: f64) -> f64 {
    f64::from(day).mul_add(f64::from(MS_PER_DAY), time).floor()
}

/// `EpochTimeToDayNumber`
///
/// This equation is the equivalent to `ECMAScript`'s `Date(t)`
pub(crate) fn epoch_time_to_day_number(t: f64) -> i32 {
    (t / f64::from(MS_PER_DAY)).floor() as i32
}

/// Mathematically determine the days in a year.
pub(crate) fn mathematical_days_in_year(y: i32) -> i32 {
    if y % 4 != 0 {
        365
    } else if y % 4 == 0 && y % 100 != 0 {
        366
    } else if y % 100 == 0 && y % 400 != 0 {
        365
    } else {
        // Assert that y is divisble by 400 to ensure we are returning the correct result.
        assert_eq!(y % 400, 0);
        366
    }
}

/// Returns the epoch day number for a given year.
pub(crate) fn epoch_day_number_for_year(y: f64) -> f64 {
    365.0f64.mul_add(y - 1970.0, ((y - 1969.0) / 4.0).floor()) - ((y - 1901.0) / 100.0).floor()
        + ((y - 1601.0) / 400.0).floor()
}

pub(crate) fn epoch_time_for_year(y: i32) -> f64 {
    f64::from(MS_PER_DAY) * epoch_day_number_for_year(f64::from(y))
}

pub(crate) fn epoch_time_to_epoch_year(t: f64) -> i32 {
    // roughly calculate the largest possible year given the time t,
    // then check and refine the year.
    let day_count = epoch_time_to_day_number(t);
    let mut year = (day_count / 365) + 1970;
    loop {
        if epoch_time_for_year(year) <= t {
            break;
        }
        year -= 1;
    }

    year
}

/// Returns either 1 (true) or 0 (false)
pub(crate) fn mathematical_in_leap_year(t: f64) -> i32 {
    mathematical_days_in_year(epoch_time_to_epoch_year(t)) - 365
}

pub(crate) fn epoch_time_to_month_in_year(t: f64) -> u8 {
    const DAYS: [i32; 11] = [30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333];
    const LEAP_DAYS: [i32; 11] = [30, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

    let in_leap_year = mathematical_in_leap_year(t) == 1;
    let day = epoch_time_to_day_in_year(t);

    let result = if in_leap_year {
        LEAP_DAYS.binary_search(&day)
    } else {
        DAYS.binary_search(&day)
    };

    match result {
        Ok(i) | Err(i) => i as u8,
    }
}

// Returns the time for a month in a given year plus date(t) = 1.
pub(crate) fn epoch_time_for_month_given_year(m: i32, y: i32) -> f64 {
    let leap_day = mathematical_days_in_year(y) - 365;

    // Includes day. i.e. end of month + 1
    let days = match m {
        0 => 0,
        1 => 31,
        2 => 59 + leap_day,
        3 => 90 + leap_day,
        4 => 120 + leap_day,
        5 => 151 + leap_day,
        6 => 181 + leap_day,
        7 => 212 + leap_day,
        8 => 243 + leap_day,
        9 => 273 + leap_day,
        10 => 304 + leap_day,
        11 => 334 + leap_day,
        _ => unreachable!(),
    };

    f64::from(MS_PER_DAY) * f64::from(days)
}

pub(crate) fn epoch_time_to_date(t: f64) -> u8 {
    const OFFSETS: [i16; 12] = [
        1, -30, -58, -89, -119, -150, -180, -211, -242, -272, -303, -333,
    ];
    let day_in_year = epoch_time_to_day_in_year(t);
    let in_leap_year = mathematical_in_leap_year(t);
    let month = epoch_time_to_month_in_year(t);

    // Cast from i32 to usize should be safe as the return must be 0-11
    let mut date = day_in_year + i32::from(OFFSETS[month as usize]);

    if month >= 2 {
        date -= in_leap_year;
    }

    // This return of date should be < 31.
    date as u8
}

pub(crate) fn epoch_time_to_day_in_year(t: f64) -> i32 {
    epoch_time_to_day_number(t)
        - (epoch_day_number_for_year(f64::from(epoch_time_to_epoch_year(t))) as i32)
}

// EpochTimeTOWeekDay -> REMOVED

// ==== End Date Equations ====

// ==== Begin Calendar Equations ====

// NOTE: below was the iso methods in temporal::calendar -> Need to be reassessed.

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28 + mathematical_in_leap_year(epoch_time_for_year(year)),
        _ => unreachable!("ISODaysInMonth panicking is an implementation error."),
    }
}

// The below calendar abstract equations/utilities were removed for being unused.
// 12.2.32 `ToISOWeekOfYear ( year, month, day )`
// 12.2.33 `ISOMonthCode ( month )`
// 12.2.39 `ToISODayOfYear ( year, month, day )`
// 12.2.40 `ToISODayOfWeek ( year, month, day )`

// ==== End Calendar Equations ====

// ==== Tests =====

// TODO(nekevss): Add way more to the below.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_to_month() {
        let oct_2023 = 1_696_459_917_000_f64;
        let mar_1_2020 = 1_583_020_800_000_f64;
        let feb_29_2020 = 1_582_934_400_000_f64;
        let mar_1_2021 = 1_614_556_800_000_f64;

        assert_eq!(epoch_time_to_month_in_year(oct_2023), 9);
        assert_eq!(epoch_time_to_month_in_year(mar_1_2020), 2);
        assert_eq!(mathematical_in_leap_year(mar_1_2020), 1);
        assert_eq!(epoch_time_to_month_in_year(feb_29_2020), 1);
        assert_eq!(mathematical_in_leap_year(feb_29_2020), 1);
        assert_eq!(epoch_time_to_month_in_year(mar_1_2021), 2);
        assert_eq!(mathematical_in_leap_year(mar_1_2021), 0);
    }

    #[test]
    fn time_for_month_and_year() {
        // NOTE: Month is 0-11

        // Test standard year.
        let standard_year_t = epoch_time_for_year(2015);
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(0, 2015)),
            1,
            "January is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(1, 2015)),
            1,
            "February is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(2, 2015)),
            1,
            "March is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(3, 2015)),
            1,
            "April is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(4, 2015)),
            1,
            "May is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(5, 2015)),
            1,
            "June is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(6, 2015)),
            1,
            "July is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(7, 2015)),
            1,
            "August is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(8, 2015)),
            1,
            "September is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(9, 2015)),
            1,
            "October is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(10, 2015)),
            1,
            "November is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(standard_year_t + epoch_time_for_month_given_year(11, 2015)),
            1,
            "December is unaligned."
        );

        // Test leap Year
        let leap_year_t = epoch_time_for_year(2020);
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(0, 2020)),
            1,
            "January is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(1, 2020)),
            1,
            "February is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(2, 2020)),
            1,
            "March is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(3, 2020)),
            1,
            "April is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(4, 2020)),
            1,
            "May is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(5, 2020)),
            1,
            "June is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(6, 2020)),
            1,
            "July is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(7, 2020)),
            1,
            "August is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(8, 2020)),
            1,
            "September is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(9, 2020)),
            1,
            "October is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(10, 2020)),
            1,
            "November is unaligned."
        );
        assert_eq!(
            epoch_time_to_date(leap_year_t + epoch_time_for_month_given_year(11, 2020)),
            1,
            "December is unaligned."
        );
    }
}
