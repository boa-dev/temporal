//! `timezone_provider` is the core data provider implementations for `temporal_rs`

// What are we even doing here? Why are providers needed?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

use alloc::borrow::Cow;

#[cfg(any(feature = "tzif", feature = "zoneinfo64"))]
use crate::provider::TimeZoneProviderResult;
#[cfg(any(feature = "tzif", feature = "zoneinfo64"))]
use crate::TimeZoneProviderError;
#[cfg(any(feature = "tzif", feature = "zoneinfo64"))]
use crate::SINGLETON_IANA_NORMALIZER;
use zerotrie::ZeroAsciiIgnoreCaseTrie;
use zerovec::{VarZeroVec, ZeroVec};

#[cfg(feature = "datagen")]
pub(crate) mod datagen;

/// A data struct for IANA identifier normalization
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(
    feature = "datagen",
    derive(serde::Serialize, yoke::Yokeable, serde::Deserialize, databake::Bake)
)]
#[cfg_attr(feature = "datagen", databake(path = timezone_provider))]
pub struct IanaIdentifierNormalizer<'data> {
    /// TZDB version
    pub version: Cow<'data, str>,
    /// An index to the location of the normal identifier.
    #[cfg_attr(feature = "datagen", serde(borrow))]
    pub available_id_index: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,
    /// A "links" table mapping non-canonical IDs to their canonical IDs
    #[cfg_attr(feature = "datagen", serde(borrow))]
    pub non_canonical_identifiers: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,

    /// The normalized IANA identifier
    #[cfg_attr(feature = "datagen", serde(borrow))]
    pub normalized_identifiers: VarZeroVec<'data, str>,
}

#[cfg(any(feature = "tzif", feature = "zoneinfo64"))]
pub(crate) fn normalize_identifier_with_compiled(
    identifier: &[u8],
) -> TimeZoneProviderResult<Cow<'static, str>> {
    if let Some(index) = SINGLETON_IANA_NORMALIZER.available_id_index.get(identifier) {
        return SINGLETON_IANA_NORMALIZER
            .normalized_identifiers
            .get(index)
            .map(Cow::Borrowed)
            .ok_or(TimeZoneProviderError::Range("Unknown time zone identifier"));
    }

    Err(TimeZoneProviderError::Range("Unknown time zone identifier"))
}

#[cfg(any(feature = "tzif", feature = "zoneinfo64"))]
pub(crate) fn canonicalize_identifier_with_compiled(
    identifier: &[u8],
) -> TimeZoneProviderResult<Cow<'static, str>> {
    let idx = SINGLETON_IANA_NORMALIZER
        .non_canonical_identifiers
        .get(identifier)
        .or(SINGLETON_IANA_NORMALIZER.available_id_index.get(identifier));

    if let Some(index) = idx {
        return SINGLETON_IANA_NORMALIZER
            .normalized_identifiers
            .get(index)
            .map(Cow::Borrowed)
            .ok_or(TimeZoneProviderError::Range("Unknown time zone identifier"));
    }

    Err(TimeZoneProviderError::Range("Unknown time zone identifier"))
}
