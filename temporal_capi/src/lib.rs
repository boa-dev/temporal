#![allow(unused)] // Until we add all the APIs
#![warn(unused_imports)] // But we want to clean up imports
#![allow(clippy::needless_lifetimes)] // Diplomat requires explicit lifetimes at times

mod calendar;
mod error;
mod options;

mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
