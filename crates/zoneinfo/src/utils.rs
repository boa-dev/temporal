//! Utiltiy functions

// TODO: potentially make time utils reusable

pub const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

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

pub(crate) fn num_leap_days(year: i32) -> i32 {
    mathematical_days_in_year(year) - 365
}

pub(crate) fn month_to_day(m: u8, leap_day: i32) -> i32 {
    match m {
        1 => 0,
        2 => 31,
        3 => 59 + leap_day,
        4 => 90 + leap_day,
        5 => 120 + leap_day,
        6 => 151 + leap_day,
        7 => 181 + leap_day,
        8 => 212 + leap_day,
        9 => 243 + leap_day,
        10 => 273 + leap_day,
        11 => 304 + leap_day,
        12 => 334 + leap_day,
        // Sort of hack to return end of month days
        13 => 365 + leap_day,
        _ => unreachable!(),
    }
}

pub(crate) fn epoch_days_for_year(y: i32) -> i32 {
    365 * (y - 1970) + (y - 1969).div_euclid(4) - (y - 1901).div_euclid(100)
        + (y - 1601).div_euclid(400)
}

pub(crate) fn epoch_seconds_for_epoch_days(epoch_days: i32) -> i64 {
    epoch_days as i64 * SECONDS_PER_DAY
}

// The weekday number for 1970-1-1
const EPOCH_DAY_OF_WEEK_CONSTANT: i32 = 4;

pub(crate) fn epoch_days_to_week_day(epoch_days: i32) -> u8 {
    // Wrapping here will be off by 1. (Sunday = 7; Mon = 1)
    (epoch_days + EPOCH_DAY_OF_WEEK_CONSTANT).rem_euclid(7) as u8
}

/// Returns the calendar year containing the given epoch seconds.
/// Mirrors `timezone_provider::utils::epoch_time_to_iso_year` (Neri-Schneider algorithm),
/// adapted for seconds rather than milliseconds.
pub(crate) fn year_from_epoch_seconds(secs: i64) -> i32 {
    const EPOCH_COMPUTATIONAL_RATA_DIE: i32 = 719_468;
    const DAYS_IN_A_400Y_CYCLE: u32 = 146_097;
    const SHIFT_CONSTANT: i32 = 3_670;
    const TWO_POWER_THIRTY_NINE: u64 = 549_755_813_888;

    let epoch_days = secs.div_euclid(SECONDS_PER_DAY) as i32;
    let rata_die = (epoch_days
        + EPOCH_COMPUTATIONAL_RATA_DIE
        + DAYS_IN_A_400Y_CYCLE as i32 * SHIFT_CONSTANT) as u32;
    let shift_constant = 400 * SHIFT_CONSTANT;
    let n1 = 4 * rata_die + 3;
    let century = n1.div_euclid(DAYS_IN_A_400Y_CYCLE);
    let n2 = n1.rem_euclid(DAYS_IN_A_400Y_CYCLE) | 3;
    let century_year = (376_287_347_u64 * n2 as u64).div_euclid(TWO_POWER_THIRTY_NINE) as u32;
    let comp_year = 100 * century + century_year;
    let day_of_year = (n2 - 1461 * century_year).div_euclid(4);
    let j = (day_of_year >= 306) as u32;
    (comp_year + j) as i32 - shift_constant
}
