/// Gregorian Date Calculations
///
/// This module contains the logic for Gregorian Date Calculations.
///
/// ## Extending Neri-Schneider shift window
///
/// In there paper, Neri-Schneider calculated for a Rata Die shift
/// of 82, but that was not sufficient in order to support `Temporal`'s
/// date range, so below is a small addendum table on extending the date
/// range from s = 82 to s = 680.
///
/// | Significant Date | Computational Rata Die | Rata Die Shift
/// | -----------------|------------------------|-----------------|
/// | April 19, -271_821 | -99,280,532 | 65,429 |
/// | January 1, 1970 | 719,468 | 100,065,428 |
/// | September 14, 275,760 | 100_719_469 | 200,065,429 |
///
pub mod neri_schneider;

pub use neri_schneider::{
    gregorian_day as day, gregorian_month as month, gregorian_year as year,
    gregorian_ymd_from_epoch_days as ymd_from_epoch_days, rata_die_for_epoch_days,
    rata_die_from_gregorian_date,
};
