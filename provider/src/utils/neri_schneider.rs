//! Gregorian Date Calculations
//!
//! This module contains the logic for Gregorian Date Calculations based
//! off Cassio Neri and Lorenz Schneider's paper, [Euclidean affine functions
//! and their application to calendar algorithms][eaf-calendar-algorithms].
//!
//! ## General Usage Note
//!
//! Unless specified, Rata Die refers to the computational rata die as referenced
//! in the paper.
//!
//! ## Extending Neri-Schneider shift window
//! Temporal must support the year range [-271_821, 275_760]
//!
//! This means the epoch day range must be epoch_days.abs() <= 100_000_001
//!
//! Neri-Schneider mention shifting for a range of 32_767, so the shift
//! will need to be much greater.
//!
//! (-271_821 / 400).ciel() = s // 680
//!
//! In their paper, Neri and Schneider calculated for a Rata Die cycle
//! shift of constant of 82, but that was not sufficient in order to
//! support Temporal's date range, so below is a small addendum table
//! on extending the date range from a cyle shift of 82 to 680 in order
//! to accomodate Temporal's range.
//!
//! | Significant Date | Computational Rata Die | Rata Die Shift
//! | -----------------|------------------------|-----------------|
//! | April 19, -271_821 | -99,280,532 | 65,429 |
//! | January 1, 1970 | 719,468 | 100,065,428 |
//! | September 14, 275,760 | 100_719_469 | 200,065,429 |
//!
//! However, this shift has also been implemented by Cassio Neri, who
//! recommends using a [shift of 3670][neri-shift-context] which places the Epoch in the
//! center of the shift
//!
//! [neri-shift-context]: https://hg.mozilla.org/integration/autoland/rev/54ebf8bd2e11#l3.70
//! [eaf-calendar-algorithms]: https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172

pub const EPOCH_COMPUTATIONAL_RATA_DIE: i32 = 719_468;
pub const DAYS_IN_A_400Y_CYCLE: u32 = 146_097;

const TWO_POWER_THIRTY_NINE: u64 = 549_755_813_888; // 2^39 constant
const TWO_POWER_SIXTEEN: u32 = 65_536; // 2^16 constant

const SHIFT_CONSTANT: i32 = 3670;

#[cfg(test)]
const SHIFT_CONSTANT_EXTENDED: i64 = 5_368_710;

/// Calculate Rata Die value from gregorian
#[cfg(test)]
pub const fn epoch_days_from_gregorian_date(year: i32, month: u8, day: u8) -> i64 {
    let shift =
        SHIFT_CONSTANT_EXTENDED * DAYS_IN_A_400Y_CYCLE as i64 + EPOCH_COMPUTATIONAL_RATA_DIE as i64;
    let (comp_year, comp_month, comp_day, century) = rata_die_first_equations(year, month, day);
    let y_star = 1461 * comp_year / 4 - century + century / 4;
    let m_star = (979 * comp_month - 2919) / 32;
    (y_star as i64 + m_star + comp_day) - shift
}

// Returns Y, M, D, C
#[cfg(test)]
const fn rata_die_first_equations(year: i32, month: u8, day: u8) -> (u64, i64, i64, u64) {
    let j = (month <= 2) as i64;
    let computational_year = (year as i64 + 400 * SHIFT_CONSTANT_EXTENDED) - j;
    let computation_month = month as i64 + 12 * j;
    let computation_day = day as i64 - 1;
    (
        computational_year as u64,
        computation_month,
        computation_day,
        computational_year as u64 / 100,
    )
}

// Computational days to gregorian YMD

// Determine j
const fn j(rata_die: u32) -> u32 {
    (computational_day_of_year(rata_die) >= 306) as u32
}

const fn n_one(rata_die: u32) -> u32 {
    4 * rata_die + 3
}

const fn n_two(rata_die: u32) -> u32 {
    century_rem(rata_die) | 3
}

const fn n_three(rata_die: u32) -> u32 {
    2141 * computational_day_of_year(rata_die) + 197_913
}

const fn century_rem(rata_die: u32) -> u32 {
    n_one(rata_die).rem_euclid(DAYS_IN_A_400Y_CYCLE)
}

pub const fn century_number(rata_die: u32) -> u32 {
    n_one(rata_die).div_euclid(DAYS_IN_A_400Y_CYCLE)
}

// Z
pub const fn computational_year_of_century(rata_die: u32) -> u64 {
    (376_287_347 * n_two(rata_die) as u64).div_euclid(TWO_POWER_THIRTY_NINE)
}

// N_y
pub const fn computational_day_of_year(rata_die: u32) -> u32 {
    (n_two(rata_die) - 1461 * computational_year_of_century(rata_die) as u32).div_euclid(4)
}

// Y
pub const fn computational_year(rata_die: u32) -> u32 {
    100 * century_number(rata_die) + computational_year_of_century(rata_die) as u32
}

pub const fn computational_month(rata_die: u32) -> u32 {
    n_three(rata_die).div_euclid(TWO_POWER_SIXTEEN)
}

pub const fn year(computational_rata_die: u32, shift_constant: i32) -> i32 {
    (computational_year(computational_rata_die) + j(computational_rata_die)) as i32 - shift_constant
}

pub const fn month(compulational_rata_die: u32) -> u8 {
    (computational_month(compulational_rata_die) - 12 * j(compulational_rata_die)) as u8
}

/// Get the computational Rata Die for given Epoch Days with the cycle shiftc.
///
/// For more on `cycle_shifts`, see [`ymd_from_epoch_days`]
pub const fn rata_die_for_epoch_days(epoch_days: i32) -> (u32, i32) {
    let rata_die = (epoch_days
        + EPOCH_COMPUTATIONAL_RATA_DIE
        + DAYS_IN_A_400Y_CYCLE as i32 * SHIFT_CONSTANT) as u32; // epoch_days + K
    (rata_die, 400 * SHIFT_CONSTANT)
}

// ==== Unit tests ====
// For unit tests, see `temporal_rs`
