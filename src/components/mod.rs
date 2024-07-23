//! The primary date-time components provided by Temporal.
//!
//! The below components are the main primitives of the `Temporal` specification:
//!   - `Date` -> `PlainDate`
//!   - `DateTime` -> `PlainDateTime`
//!   - `Time` -> `PlainTime`
//!   - `Duration` -> `Duration`
//!   - `Instant` -> `Instant`
//!   - `MonthDay` -> `PlainMonthDay`
//!   - `YearMonth` -> `PlainYearMonth`
//!   - `ZonedDateTime` -> `ZonedDateTime`
//!
//! The Temporal specification, along with this implementation aims to provide
//! full support for time zones and non-gregorian calendars that are compliant
//! with standards like ISO 8601, RFC 3339, and RFC 5545.

// TODO: Expand upon above introduction.

pub mod calendar;
pub mod duration;
pub mod tz;

mod date;
mod datetime;
mod instant;
mod month_day;
mod time;
mod year_month;
mod zoneddatetime;

use std::str::FromStr;

#[doc(inline)]
pub use date::{Date, PartialDate};
#[doc(inline)]
pub use datetime::DateTime;
#[doc(inline)]
pub use duration::Duration;
#[doc(inline)]
pub use instant::Instant;
#[doc(inline)]
pub use month_day::MonthDay;
#[doc(inline)]
pub use time::Time;
#[doc(inline)]
pub use year_month::YearMonth;
pub use year_month::YearMonthFields;
#[doc(inline)]
pub use zoneddatetime::ZonedDateTime;

use crate::TemporalError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MonthCode {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
}

impl MonthCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::One => "M01",
            Self::Two => "M02",
            Self::Three => "M03",
            Self::Four => "M04",
            Self::Five => "M05",
            Self::Six => "M06",
            Self::Seven => "M07",
            Self::Eight => "M08",
            Self::Nine => "M09",
            Self::Ten => "M10",
            Self::Eleven => "M11",
            Self::Twelve => "M12",
            Self::Thirteen => "M13",
        }
    }
}

impl FromStr for MonthCode {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "M01" => Ok(Self::One),
            "M02" => Ok(Self::Two),
            "M03" => Ok(Self::Three),
            "M04" => Ok(Self::Four),
            "M05" => Ok(Self::Five),
            "M06" => Ok(Self::Six),
            "M07" => Ok(Self::Seven),
            "M08" => Ok(Self::Eight),
            "M09" => Ok(Self::Nine),
            "M10" => Ok(Self::Ten),
            "M11" => Ok(Self::Eleven),
            "M12" => Ok(Self::Twelve),
            "M13" => Ok(Self::Thirteen),
            _ => {
                Err(TemporalError::range()
                    .with_message("monthCode is not within the valid values."))
            }
        }
    }
}

impl TryFrom<u8> for MonthCode {
    type Error = TemporalError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            10 => Ok(Self::Ten),
            11 => Ok(Self::Eleven),
            12 => Ok(Self::Twelve),
            13 => Ok(Self::Thirteen),
            _ => Err(TemporalError::range().with_message("Invalid MonthCode value.")),
        }
    }
}
