#![allow(unused)] // Until we add all the APIs
#![warn(unused_imports)] // But we want to clean up imports
#![allow(clippy::needless_lifetimes)] // Diplomat requires explicit lifetimes at times
#![allow(clippy::too_many_arguments)] // We're mapping APIs with the same argument size
#![allow(clippy::wrong_self_convention)] // Diplomat forces self conventions that may not always be ideal

mod calendar;
mod duration;
mod error;
mod options;

mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
