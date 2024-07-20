//! This module implements a native Rust `TemporalField` and components.

use core::fmt;
use std::str::FromStr;

use crate::{components::calendar::Calendar, error::TemporalError, iso::IsoDate, TemporalResult};

use bitflags::bitflags;
use tinystr::TinyAsciiStr;

// use rustc_hash::FxHashSet;
bitflags! {
    /// FieldMap maps the currently active fields on the `TemporalField`
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct FieldMap: u16 {
        /// Represents an active `year` field
        const YEAR = 0b0000_0000_0000_0001;
        /// Represents an active `month` field
        const MONTH = 0b0000_0000_0000_0010;
        /// Represents an active `monthCode` field
        const MONTH_CODE = 0b0000_0000_0000_0100;
        /// Represents an active `day` field
        const DAY = 0b0000_0000_0000_1000;
        /// Represents an active `hour` field
        const HOUR = 0b0000_0000_0001_0000;
        /// Represents an active `minute` field
        const MINUTE = 0b0000_0000_0010_0000;
        /// Represents an active `second` field
        const SECOND = 0b0000_0000_0100_0000;
        /// Represents an active `millisecond` field
        const MILLISECOND = 0b0000_0000_1000_0000;
        /// Represents an active `microsecond` field
        const MICROSECOND = 0b0000_0001_0000_0000;
        /// Represents an active `nanosecond` field
        const NANOSECOND = 0b0000_0010_0000_0000;
        /// Represents an active `offset` field
        const OFFSET = 0b0000_0100_0000_0000;
        /// Represents an active `era` field
        const ERA = 0b0000_1000_0000_0000;
        /// Represents an active `eraYear` field
        const ERA_YEAR = 0b0001_0000_0000_0000;
        /// Represents an active `timeZone` field
        const TIME_ZONE = 0b0010_0000_0000_0000;
        // NOTE(nekevss): Two bits preserved if needed.
    }
}

impl From<FieldKey> for FieldMap {
    #[inline]
    fn from(value: FieldKey) -> Self {
        match value {
            FieldKey::Year => FieldMap::YEAR,
            FieldKey::Month => FieldMap::MONTH,
            FieldKey::MonthCode => FieldMap::MONTH_CODE,
            FieldKey::Day => FieldMap::DAY,
            FieldKey::Hour => FieldMap::HOUR,
            FieldKey::Minute => FieldMap::MINUTE,
            FieldKey::Second => FieldMap::SECOND,
            FieldKey::Millisecond => FieldMap::MILLISECOND,
            FieldKey::Microsecond => FieldMap::MICROSECOND,
            FieldKey::Nanosecond => FieldMap::NANOSECOND,
            FieldKey::Offset => FieldMap::OFFSET,
            FieldKey::Era => FieldMap::ERA,
            FieldKey::EraYear => FieldMap::ERA_YEAR,
            FieldKey::TimeZone => FieldMap::TIME_ZONE,
        }
    }
}

/// The post conversion field value.
#[derive(Debug, Clone)]
pub enum FieldValue {
    /// Designates the values as an integer.
    Integer(Option<i32>),
    /// Designates the value as a string.
    String(String),
}

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        Self::Integer(Some(value))
    }
}

/// The Conversion type of a field.
#[derive(Debug, Clone, Copy)]
pub enum FieldConversion {
    /// Designates the Conversion type is `ToIntegerWithTruncation`
    ToIntegerWithTruncation,
    /// Designates the Conversion type is `ToPositiveIntegerWithTruncation`
    ToPositiveIntegerWithTruncation,
    /// Designates the Conversion type is `ToPrimitiveRequireString`
    ToPrimativeAndRequireString,
    /// Designates the Conversion type is nothing
    None,
}

impl FromStr for FieldConversion {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "year" | "hour" | "minute" | "second" | "millisecond" | "microsecond"
            | "nanosecond" => Ok(Self::ToIntegerWithTruncation),
            "month" | "day" => Ok(Self::ToPositiveIntegerWithTruncation),
            "monthCode" | "offset" | "eraYear" => Ok(Self::ToPrimativeAndRequireString),
            _ => Err(TemporalError::range()
                .with_message(format!("{s} is not a valid TemporalField Property"))),
        }
    }
}

/// This enum represents the valid keys of a `TemporalField`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKey {
    Year,
    Month,
    MonthCode,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Offset,
    Era,
    EraYear,
    TimeZone,
}

impl TryFrom<FieldMap> for FieldKey {
    type Error = TemporalError;
    fn try_from(value: FieldMap) -> Result<Self, Self::Error> {
        match value {
            FieldMap::YEAR => Ok(FieldKey::Year),
            FieldMap::MONTH => Ok(FieldKey::Month),
            FieldMap::MONTH_CODE => Ok(FieldKey::MonthCode),
            FieldMap::DAY => Ok(FieldKey::Day),
            FieldMap::HOUR => Ok(FieldKey::Hour),
            FieldMap::MINUTE => Ok(FieldKey::Minute),
            FieldMap::SECOND => Ok(FieldKey::Second),
            FieldMap::MILLISECOND => Ok(FieldKey::Millisecond),
            FieldMap::MICROSECOND => Ok(FieldKey::Microsecond),
            FieldMap::NANOSECOND => Ok(FieldKey::Nanosecond),
            FieldMap::OFFSET => Ok(FieldKey::Offset),
            FieldMap::ERA => Ok(FieldKey::Era),
            FieldMap::ERA_YEAR => Ok(FieldKey::EraYear),
            FieldMap::TIME_ZONE => Ok(FieldKey::TimeZone),
            _ => Err(TemporalError::range().with_message("Invalid FieldMap bit value.")),
        }
    }
}

impl FromStr for FieldKey {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "year" => Ok(Self::Year),
            "month" => Ok(Self::Month),
            "monthCode" => Ok(Self::MonthCode),
            "day" => Ok(Self::Day),
            "hour" => Ok(Self::Hour),
            "minute" => Ok(Self::Minute),
            "second" => Ok(Self::Second),
            "millisecond" => Ok(Self::Millisecond),
            "microsecond" => Ok(Self::Microsecond),
            "nanosecond" => Ok(Self::Nanosecond),
            "offset" => Ok(Self::Offset),
            "era" => Ok(Self::Era),
            "eraYear" => Ok(Self::EraYear),
            "timeZone" => Ok(Self::TimeZone),
            _ => Err(TemporalError::general(
                "String cannot be converted to FieldKey",
            )),
        }
    }
}

/// `TemporalFields` acts as a native Rust implementation of the `fields` object
///
/// The temporal fields are laid out in the Temporal proposal under section 13.46 `PrepareTemporalFields`
/// with conversion and defaults laid out by Table 17 (displayed below).
///
/// ## Table 17: Temporal field requirements
///
/// |   Property   |           Conversion              |  Default   |
/// | -------------|-----------------------------------|------------|
/// | "year"       |     `ToIntegerWithTruncation`     | undefined  |
/// | "month"      | `ToPositiveIntegerWithTruncation` | undefined  |
/// | "monthCode"  |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "day"        | `ToPositiveIntegerWithTruncation` | undefined  |
/// | "hour"       |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "minute"     |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "second"     |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "millisecond"|     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "microsecond"|     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "nanosecond" |     `ToIntegerWithTruncation`     |    +0𝔽     |
/// | "offset"     |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "era"        |   `ToPrimitiveAndRequireString`   | undefined  |
/// | "eraYear"    |     `ToIntegerWithTruncation`     | undefined  |
/// | "timeZone"   |              `None`               | undefined  |
#[derive(Debug, Default, Clone)]
pub struct TemporalFields {
    bit_map: FieldMap,
    pub(crate) year: Option<i32>,
    pub(crate) month: Option<i32>,
    pub(crate) month_code: Option<TinyAsciiStr<4>>,
    pub(crate) day: Option<i32>,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
    offset: Option<TinyAsciiStr<16>>,
    pub(crate) era: Option<TinyAsciiStr<16>>,
    era_year: Option<i32>,
    time_zone: Option<TinyAsciiStr<32>>,
}

impl TemporalFields {
    /// Returns an iterator over the `TemporalField`'s keys.
    #[inline]
    pub fn keys(&self) -> TemporalFieldsKeys {
        TemporalFieldsKeys {
            iter: self.bit_map.iter(),
        }
    }

    /// Returns an iterator over the `TemporalField`'s values.
    #[inline]
    pub fn values(&self) -> Values {
        Values {
            fields: self,
            iter: self.bit_map.iter(),
        }
    }

    /// Gets the value of a `FieldKey` if the field has been set to active. If the field
    /// has not been set, then return `None`.
    #[inline]
    pub fn get(&self, key: FieldKey) -> Option<FieldValue> {
        if !self.bit_map.contains(key.into()) {
            return None;
        }

        match key {
            FieldKey::Year => Some(FieldValue::Integer(self.year)),
            FieldKey::Month => Some(FieldValue::Integer(self.month)),
            FieldKey::MonthCode => Some(FieldValue::String(
                self.month_code.map_or(String::default(), |s| s.to_string()),
            )),
            FieldKey::Day => Some(FieldValue::Integer(self.day)),
            FieldKey::Hour => Some(FieldValue::from(self.hour)),
            FieldKey::Minute => Some(FieldValue::from(self.minute)),
            FieldKey::Second => Some(FieldValue::from(self.second)),
            FieldKey::Millisecond => Some(FieldValue::from(self.millisecond)),
            FieldKey::Microsecond => Some(FieldValue::from(self.microsecond)),
            FieldKey::Nanosecond => Some(FieldValue::from(self.nanosecond)),
            FieldKey::Offset => Some(FieldValue::String(
                self.offset.map_or(String::default(), |s| s.to_string()),
            )),
            FieldKey::Era => Some(FieldValue::String(
                self.era.map_or(String::default(), |s| s.to_string()),
            )),
            FieldKey::EraYear => Some(FieldValue::Integer(self.era_year)),
            FieldKey::TimeZone => Some(FieldValue::String(
                self.time_zone.map_or(String::default(), |s| s.to_string()),
            )),
        }
    }

    /// Validate and insert a key-value pair. This will also set the field as acitve if the value was successfully inserted.
    #[inline]
    pub fn insert(&mut self, key: FieldKey, value: FieldValue) -> TemporalResult<()> {
        match key {
            FieldKey::Year => {
                let FieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.year = value;
            }
            FieldKey::Month => {
                let FieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.month = value;
            }
            FieldKey::MonthCode => {
                let FieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.month_code = Some(
                    TinyAsciiStr::<4>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid MonthCode id."))?,
                );
            }
            FieldKey::Day => {
                let FieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.day = value;
            }
            FieldKey::Hour => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.hour = value;
            }
            FieldKey::Minute => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.minute = value;
            }
            FieldKey::Second => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.second = value;
            }
            FieldKey::Millisecond => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.millisecond = value;
            }
            FieldKey::Microsecond => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.microsecond = value;
            }
            FieldKey::Nanosecond => {
                let FieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.nanosecond = value;
            }
            FieldKey::Offset => {
                let FieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.offset = Some(
                    TinyAsciiStr::<16>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid offset string."))?,
                );
            }
            FieldKey::Era => {
                let FieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.era = Some(
                    TinyAsciiStr::<16>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid era identifier."))?,
                );
            }
            FieldKey::EraYear => {
                let FieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.day = value;
            }
            FieldKey::TimeZone => {
                let FieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.time_zone = Some(
                    TinyAsciiStr::<32>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid Time Zone identifier."))?,
                );
            }
        }

        // Set the field as active and exit.
        self.bit_map.set(key.into(), true);
        Ok(())
    }

    /// Resolve `TemporalFields` month and monthCode fields.
    #[inline]
    pub(crate) fn iso_resolve_month(&mut self) -> TemporalResult<()> {
        let Some(mc) = self.month_code else {
            match self.month {
                Some(_) => return Ok(()),
                None => {
                    return Err(TemporalError::range()
                        .with_message("month and MonthCode values cannot both be undefined."))
                }
            };
        };

        // MonthCode is present and needs to be resolved.

        let month_code_integer = month_code_to_integer(mc)?;

        if self.month.is_some() && self.month != Some(month_code_integer) {
            return Err(
                TemporalError::range().with_message("month and monthCode cannot be resolved.")
            );
        }

        self.insert(FieldKey::Month, FieldValue::from(month_code_integer))?;

        Ok(())
    }

    // TODO: Determine if this should be moved to `Calendar`.
    /// Merges two `TemporalFields` depending on the calendar.
    #[inline]
    pub fn merge_fields(&self, other: &Self, calendar: Calendar) -> TemporalResult<Self> {
        let add_keys = other.keys().collect::<Vec<_>>();
        let overridden_keys = calendar.field_keys_to_ignore(&add_keys)?;

        let mut result = Self::default();

        for key in self.keys() {
            let value = if overridden_keys.contains(&key) {
                other.get(key)
            } else {
                self.get(key)
            };

            let Some(value) = value else {
                return Err(TemporalError::general(
                    "Nonexistent FieldKey used when merging fields.",
                ));
            };

            result.insert(key, value)?;
        }

        Ok(result)
    }
}

impl From<IsoDate> for TemporalFields {
    fn from(value: IsoDate) -> Self {
        Self {
            bit_map: FieldMap::YEAR | FieldMap::MONTH | FieldMap::DAY,
            year: Some(value.year),
            month: Some(value.month.into()),
            day: Some(value.day.into()),
            ..Default::default()
        }
    }
}

/// Iterator over `TemporalFields` keys.
pub struct TemporalFieldsKeys {
    iter: bitflags::iter::Iter<FieldMap>,
}

impl fmt::Debug for TemporalFieldsKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TemporalFields KeyIterator")
    }
}

impl Iterator for TemporalFieldsKeys {
    type Item = FieldKey;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()?.try_into().ok()
    }
}

/// An iterator over `TemporalFields`'s values.
pub struct Values<'a> {
    fields: &'a TemporalFields,
    iter: bitflags::iter::Iter<FieldMap>,
}

impl fmt::Debug for Values<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TemporalFields Values Iterator")
    }
}

impl Iterator for Values<'_> {
    type Item = FieldValue;

    fn next(&mut self) -> Option<Self::Item> {
        let field = self.iter.next()?;

        match field {
            FieldMap::YEAR => Some(FieldValue::Integer(self.fields.year)),
            FieldMap::MONTH => Some(FieldValue::Integer(self.fields.month)),
            FieldMap::MONTH_CODE => Some(FieldValue::String(
                self.fields
                    .month_code
                    .map_or(String::default(), |s| s.to_string()),
            )),
            FieldMap::DAY => Some(FieldValue::Integer(self.fields.day)),
            FieldMap::HOUR => Some(FieldValue::from(self.fields.hour)),
            FieldMap::MINUTE => Some(FieldValue::from(self.fields.minute)),
            FieldMap::SECOND => Some(FieldValue::from(self.fields.second)),
            FieldMap::MILLISECOND => Some(FieldValue::from(self.fields.millisecond)),
            FieldMap::MICROSECOND => Some(FieldValue::from(self.fields.microsecond)),
            FieldMap::NANOSECOND => Some(FieldValue::from(self.fields.nanosecond)),
            FieldMap::OFFSET => Some(FieldValue::String(
                self.fields
                    .offset
                    .map_or(String::default(), |s| s.to_string()),
            )),
            FieldMap::ERA => Some(FieldValue::String(
                self.fields.era.map_or(String::default(), |s| s.to_string()),
            )),
            FieldMap::ERA_YEAR => Some(FieldValue::Integer(self.fields.era_year)),
            FieldMap::TIME_ZONE => Some(FieldValue::String(
                self.fields
                    .time_zone
                    .map_or(String::default(), |s| s.to_string()),
            )),
            _ => None,
        }
    }
}

fn month_code_to_integer(mc: TinyAsciiStr<4>) -> TemporalResult<i32> {
    match mc.as_str() {
        "M01" => Ok(1),
        "M02" => Ok(2),
        "M03" => Ok(3),
        "M04" => Ok(4),
        "M05" => Ok(5),
        "M06" => Ok(6),
        "M07" => Ok(7),
        "M08" => Ok(8),
        "M09" => Ok(9),
        "M10" => Ok(10),
        "M11" => Ok(11),
        "M12" => Ok(12),
        "M13" => Ok(13),
        _ => Err(TemporalError::range().with_message("monthCode is not within the valid values.")),
    }
}
