//! Deprecated aliases for the parsed intermediate types.
//!
//! These types moved to [`crate::parsers::intermediates`], where `TemporalParser`
//! produces them. This module remains only as a compatibility shim.

#[deprecated(
    since = "0.2.7",
    note = "Use `parsers::intermediates::ParsedDate` instead"
)]
pub type ParsedDate = crate::parsers::intermediates::ParsedDate;

#[deprecated(
    since = "0.2.7",
    note = "Use `parsers::intermediates::ParsedDateTime` instead"
)]
pub type ParsedDateTime = crate::parsers::intermediates::ParsedDateTime;

#[deprecated(
    since = "0.2.7",
    note = "Use `parsers::intermediates::ParsedZonedDateTime` instead"
)]
pub type ParsedZonedDateTime = crate::parsers::intermediates::ParsedZonedDateTime;
