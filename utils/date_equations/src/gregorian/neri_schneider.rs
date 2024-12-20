
// NOTE: Temporal must support the year range [-271_821, 275_760]
//
// This means the epoch day range must be epoch_days.abs() <= 100_000_001
//
// Neri-Schneider mention shifting for a range of 32_767, so the shift
// will need to be much greater.
//
// (-271_821 / 400).ciel() = s // 680
//

const EPOCH_COMPUTATIONAL_RATA_DIE: i32 = 719_468;

const DAYS_IN_A_400Y_CYCLE: u32 = 146_097;
const TWO_POWER_THIRTY_NINE: u64 = 549_755_813_888; // 2^39 constant
const TWO_POWER_SIXTEEN: u32 = 65_536; // 2^16 constant
const DAYS_IN_GREGORIAN_CYCLE: i32 = DAYS_IN_A_400Y_CYCLE as i32;

// Calculate Rata Die value from gregorian

pub fn rata_die_from_gregorian_date(year: i32, month: i32, day: i32) -> i32 {
    let (comp_year, comp_month, comp_day, century) = rata_die_first_equations(year, month, day);
    let y_star = 1461 * comp_year / 4  - century + century / 4;
    let m_star = (979 * comp_month - 2919) / 32;
    y_star + m_star + comp_day
}

// Returns Y, M, D, C
fn rata_die_first_equations(year: i32, month: i32, day: i32) -> (i32, i32, i32, i32) {
    let j = (month <= 2) as i32;
    let computational_year = year - j;
    let computation_month = month + 12 * j;
    let computation_day = day - 1;
    (computational_year, computation_month, computation_day, computational_year / 100)
}

// Computational days to gregorian YMD

// Determine j
const fn j(rata_die: u32) -> u32 {
    (days_in_century(rata_die) >= 306) as u32
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

// Returns C, N_c
const fn first_equations(rata_die: u32) -> (u32, u32) {
    let n_one = n_one(rata_die);
    let century_rem = n_one.rem_euclid(146_097);
    let century_num = n_one.div_euclid(DAYS_IN_A_400Y_CYCLE);
    (century_num, century_rem)
}

const fn century_rem(rata_die: u32) -> u32 {
    n_one(rata_die).rem_euclid(DAYS_IN_A_400Y_CYCLE)
}

pub const fn century_number(rata_die: u32) -> u32 {
    n_one(rata_die).div_euclid(DAYS_IN_A_400Y_CYCLE)
}

pub const fn days_in_century(rata_die: u32) -> u32 {
    century_rem(rata_die).div_euclid(4)
}

/// returns Y, N_y
const fn second_equations(rata_die: u32) -> (u32, u32) {
    let (century, rem) = first_equations(rata_die);
    let n_two = rem | 3;
    let year_of_century = (376_287_347 * n_two as u64).div_euclid(TWO_POWER_THIRTY_NINE) as u32;
    let day_of_year = (n_two - 1461 * year_of_century).div_euclid(4);
    let year = 100 * century + year_of_century;
    (year, day_of_year)
}

// Y, M, D, N_y
const fn third_equations(rata_die: u32) -> (u32, u32, u32, u32) {
    let (year, day_of_year) = second_equations(rata_die);
    let n_three = 2141 * day_of_year + 197_913;
    let month = n_three.div_euclid(TWO_POWER_SIXTEEN);
    let day = n_three.rem_euclid(TWO_POWER_SIXTEEN).div_euclid(2141);
    (year, month, day, day_of_year)
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

pub const fn computational_day(rata_die: u32) -> u32 {
    n_three(rata_die).rem_euclid(TWO_POWER_SIXTEEN).div_euclid(2141)
}

pub const fn gregorian_year(rata_die: u32) -> u32 {
    computational_year(rata_die) + j(rata_die)
}

pub const fn gregorian_month(rata_die: u32) -> u32 {
    computational_month(rata_die) - 12 * j(rata_die)
}

pub const fn gregorian_day(rata_die: u32) -> u32 {
    computational_day(rata_die) + 1
}

pub const fn gregorian_ymd(rata_die: u32) -> (i32, u8, u8) {
    let (year, month, day, day_of_year) = third_equations(rata_die);
    let j = (day_of_year >= 306) as u32;
    let year = year + j;
    let month = month - 12 * j;
    let day = day + 1;
    (year as i32, month as u8, day as u8)
}

const SHIFTS: i32 = 680;

pub const fn gregorian_ymd_from_epoch_days(epoch_days: i32) -> (i32, u8, u8) {
    let rata_die_shift_constant = EPOCH_COMPUTATIONAL_RATA_DIE + DAYS_IN_GREGORIAN_CYCLE * SHIFTS; // K
    let year_shift_constant = 400 * SHIFTS;

    let (year, month, day) = gregorian_ymd((epoch_days + rata_die_shift_constant) as u32);
    // Shift the year back to the proper date
    (year - year_shift_constant, month, day)
}


#[cfg(test)]
mod tests {
    use super::*;

    const EPOCH_RATA_DIE: u32 = 719_468; // This is the Rata Die for 1970-01-01

    #[test]
    fn epoch_century_number() {
        let century_number = century_number(EPOCH_RATA_DIE);
        assert_eq!(century_number, 19);
        let day_number_in_century = days_in_century(EPOCH_RATA_DIE);
        assert_eq!(day_number_in_century, 25508);
    }

    #[test]
    fn epoch_year_of_century() {
        let year = computational_year_of_century(EPOCH_RATA_DIE);
        assert_eq!(year, 69);
    }

    #[test]
    fn epoch_day_of_year() {
        let day = computational_day_of_year(EPOCH_RATA_DIE);
        assert_eq!(day, 306); // Beginning of January in the computational calendar is day number 306
    }

    #[test]
    fn epoch_year() {
        let day = computational_year(EPOCH_RATA_DIE);
        assert_eq!(day, 1969);
    }

    #[test]
    fn epoch_ymd() {
        let ymd = gregorian_ymd(EPOCH_RATA_DIE);
        assert_eq!(ymd, (1970, 1, 1))
    }

    #[test]
    fn rata_die_from_date() {
        let epoch_rata_die = rata_die_from_gregorian_date(1970, 1, 1);
        assert_eq!(epoch_rata_die, 719_468);
        let neri_scneider_limit_max_rata_die = rata_die_from_gregorian_date(32767, 12, 31);
        assert_eq!(neri_scneider_limit_max_rata_die, 11_968_205);
        let neri_schneider_limit_min_rata_die = rata_die_from_gregorian_date(-32767, 1, 1);
        assert_eq!(neri_schneider_limit_min_rata_die, -11_967_960);
        let js_max_rata_die = rata_die_from_gregorian_date(275_760, 9, 14);
        assert_eq!(js_max_rata_die, 100_719_469);
        let js_min_rata_die = rata_die_from_gregorian_date(-271_821, 4, 19);
        assert_eq!(js_min_rata_die, -99_280_532);
    }

    #[test]
    fn epoch_days_limit_to_date() {
        let max_date = gregorian_ymd_from_epoch_days(100_000_001);
        assert_eq!(max_date, (275_760, 9, 14));
        let min_date = gregorian_ymd_from_epoch_days(-100_000_001);
        assert_eq!(min_date, (-271_821, 4, 19));
    }
}
