//! `timezone_provider` is the core data provider implementations for `temporal_rs`

// What are we even doing here? Why are providers needed?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

use alloc::borrow::Cow;

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
