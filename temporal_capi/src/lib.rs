#![allow(unused)] // Until we add all the APIs
#![warn(unused_imports)] // But we want to clean up imports
#![allow(clippy::needless_lifetimes)] // Diplomat requires explicit lifetimes at times
#![allow(clippy::too_many_arguments)] // We're mapping APIs with the same argument size
#![allow(clippy::wrong_self_convention)] // Diplomat forces self conventions that may not always be ideal

pub mod calendar;
pub mod duration;
pub mod error;
pub mod instant;
pub mod iso;
pub mod options;

pub mod plain_date;
pub mod plain_date_time;
pub mod plain_month_day;
pub mod plain_time;
pub mod plain_year_month;
