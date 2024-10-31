//! The primary date-time components provided by Temporal.
//!
//! The Temporal specification, along with this implementation aims to
//! provide full support for time zones and non-gregorian calendars that
//! are compliant with standards like ISO 8601, RFC 3339, and RFC 5545.

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

#[cfg(feature = "std")]
mod now;

#[cfg(feature = "std")]
#[doc(inline)]
pub use now::Now;

#[doc(inline)]
pub use date::{PartialDate, PlainDate};
#[doc(inline)]
pub use datetime::{PartialDateTime, PlainDateTime};
#[doc(inline)]
pub use duration::Duration;
#[doc(inline)]
pub use instant::Instant;
#[doc(inline)]
pub use month_day::PlainMonthDay;
#[doc(inline)]
pub use time::{PartialTime, PlainTime};
#[doc(inline)]
pub use year_month::PlainYearMonth;
#[doc(inline)]
pub use zoneddatetime::ZonedDateTime;
