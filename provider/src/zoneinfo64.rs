//! A provider for ICU's `zoneinfo64.res` bundles

use alloc::vec::Vec;
use core::{fmt::Debug, hash::Hash, marker::PhantomData};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use writeable::Writeable;
use zerovec::ZeroVec;

#[derive(Debug)]
pub struct CompiledZoneInfo64Provider<'data> {
    pub data: ZoneInfo64Data<'data>,
}

impl Default for CompiledZoneInfo64Provider<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledZoneInfo64Provider<'static> {
    pub fn new() -> Self {
        #[cfg(target_endian = "little")]
        let bytes = include_bytes!("./data/zoneinfo64/2025b/le/zoneinfo64.res");
        #[cfg(target_endian = "big")]
        let bytes = include_bytes!("./data/zoneinfo64/2025b/be/zoneinfo64.res");
        let data = resb::binary::from_bytes::<ZoneInfo64Data<'static>>(bytes)
            .expect("Error processing resource bundle file");

        Self { data }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZoneInfo64Data<'data> {
    #[serde(rename = "TZVersion")]
    #[serde(borrow)]
    pub version: ZoneInfo64String<'data>,
    #[serde(borrow)]
    pub names: Vec<ZoneInfo64String<'data>>,
    #[serde(borrow)]
    pub zones: Vec<Zone<'data>>,
    #[serde(borrow)]
    pub regions: Vec<ZoneInfo64String<'data>>,
    // NOTE (nekevss): ZoneInfo64 Rules field is ignored for now. Difficult to deal
    // with and probably not applicable to resolution anyways.
}

#[derive(Debug, Clone)]
pub enum Zone<'data> {
    Table(ZoneTable<'data>),
    Link(u32),
}

impl<'de: 'data, 'data> Deserialize<'de> for Zone<'data> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ZoneVisitor {
            marker: PhantomData,
        })
    }
}

pub struct ZoneVisitor<'data> {
    marker: PhantomData<Zone<'data>>,
}

impl<'de: 'data, 'data> Visitor<'de> for ZoneVisitor<'data> {
    type Value = Zone<'data>;
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("expecting a link or zone table")
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Zone::Link(v))
    }
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let table = ZoneTable::deserialize(de::value::MapAccessDeserializer::new(map))?;
        Ok(Zone::Table(table))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneTable<'data> {
    #[serde(borrow)]
    pub trans_pre32: Option<ZeroVec<'data, i32>>,
    #[serde(borrow)]
    pub trans: Option<ZeroVec<'data, i32>>,
    #[serde(borrow)]
    pub final_rule: Option<ZoneInfo64String<'data>>,
    pub final_raw: Option<i32>,
    pub final_year: Option<i32>,
    #[serde(borrow)]
    pub links: Option<ZeroVec<'data, i32>>,
}

#[derive(Debug, Clone, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
#[serde(transparent)]
pub struct ZoneInfo64String<'data> {
    #[serde(borrow)]
    inner: ZeroVec<'data, u16>,
}

impl Hash for ZoneInfo64String<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for val in self.inner.iter() {
            val.hash(state);
        }
    }
}

impl Writeable for ZoneInfo64String<'_> {
    fn writeable_length_hint(&self) -> writeable::LengthHint {
        writeable::LengthHint::exact(self.inner.len())
    }
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        for char in char::decode_utf16(self.inner.iter()) {
            sink.write_char(char.expect("valid char"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "zoneinfo64-provider")]
    fn zoneinfo64_resb_test() {
        use crate::zoneinfo64::CompiledZoneInfo64Provider;
        use writeable::Writeable;

        let provider = CompiledZoneInfo64Provider::new();

        assert_eq!(provider.data.version.write_to_string(), "2025b");
    }
}
