pub(crate) mod timezone;

mod date;
mod duration;
mod instant;
mod now;
pub(crate) mod options;
mod zoneddatetime;


// TODO: Remove the aliasing

pub use date::PlainDate;
pub use datetime::PlainDateTime;
pub use zoneddatetime::ZonedDateTime;
pub use duration::Duration;
pub use instant::Instant;
pub use now::Now;
pub use time::PlainTime;


pub use crate::builtins::core::{
    DateDuration, PartialDate, PartialDateTime, PartialTime, PartialZonedDateTime,
    PlainMonthDay, PlainYearMonth, TimeDuration,
    calendar,
};

mod time {
    use crate::builtins::core;
    pub struct PlainTime(pub(crate) core::PlainTime);

    impl From<core::PlainTime> for PlainTime {
        fn from(value: core::PlainTime) -> Self {
            Self(value)
        }
    }

    impl From<PlainTime> for core::PlainTime {
        fn from(value: PlainTime) -> Self {
            value.0
        }
    }
}

mod datetime {
    use crate::builtins::core;
    pub struct PlainDateTime(pub(crate) core::PlainDateTime);

    impl From<core::PlainDateTime> for PlainDateTime {
        fn from(value: core::PlainDateTime) -> Self {
            Self(value)
        }
    }
}

