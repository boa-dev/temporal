//! Utility date and time equations for Temporal

use num_traits::{AsPrimitive, FromPrimitive};

use crate::{TemporalError, TemporalResult, MS_PER_DAY};

// NOTE: Review the below for optimizations and add ALOT of tests.

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteF64(pub(crate) f64);

impl FiniteF64 {
    #[inline]
    #[must_use]
    pub unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    pub fn as_inner(&self) -> f64 {
        self.0
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    #[inline]
    pub fn negate(&self) -> Self {
        if !self.is_zero() {
            Self(self.0 * -1.0)
        } else {
            *self
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn checked_add(&self, other: &Self) -> TemporalResult<Self> {
        let result = Self(self.0 + other.0);
        if !result.0.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(result)
    }

    #[inline]
    pub fn checked_mul_add(&self, a: FiniteF64, b: FiniteF64) -> TemporalResult<Self> {
        let result = Self(self.0.mul_add(a.0, b.0));
        if !result.0.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(result)
    }

    #[inline]
    pub(crate) fn unchecked_mul(&self, other: f64) -> Self {
        Self(self.0 * other)
    }

    pub(crate) fn as_date_value(&self) -> TemporalResult<i32> {
        if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&self.0) {
            return Err(TemporalError::range().with_message("number exceeds a valid date value."));
        }
        Ok(self.0 as i32)
    }
}

impl AsPrimitive<i64> for FiniteF64 {
    fn as_(self) -> i64 {
        self.0 as i64
    }
}

impl AsPrimitive<i128> for FiniteF64 {
    fn as_(self) -> i128 {
        self.0 as i128
    }
}

impl TryFrom<f64> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(unsafe { Self::new_unchecked(value) })
    }
}

impl TryFrom<i128> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let result = f64::from_i128(value)
            .ok_or(TemporalError::range().with_message("days exceeded a valid range."))?;
        if !result.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(unsafe { Self::new_unchecked(result) })
    }
}

impl From<i8> for FiniteF64 {
    fn from(value: i8) -> Self {
        unsafe { Self::new_unchecked(f64::from(value)) }
    }
}

impl From<i32> for FiniteF64 {
    fn from(value: i32) -> Self {
        unsafe { Self::new_unchecked(f64::from(value)) }
    }
}

impl From<u8> for FiniteF64 {
    fn from(value: u8) -> Self {
        unsafe { Self::new_unchecked(f64::from(value)) }
    }
}

impl From<u16> for FiniteF64 {
    fn from(value: u16) -> Self {
        unsafe { Self::new_unchecked(f64::from(value)) }
    }
}

impl From<u32> for FiniteF64 {
    fn from(value: u32) -> Self {
        unsafe { Self::new_unchecked(f64::from(value)) }
    }
}

impl PartialEq<f64> for FiniteF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<f64> for FiniteF64 {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
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

// Trait implementations

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
