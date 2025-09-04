//! The `TimeZoneProvider` trait.

pub use timezone_provider::provider::{
    CandidateEpochNanoseconds, EpochNanosecondsAndOffset, GapEntryOffsets, NeverProvider,
    ParseDirectionError, TimeZoneId, TimeZoneProvider, TransitionDirection, UtcOffsetSeconds,
};
