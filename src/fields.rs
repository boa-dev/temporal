//! This module implements a native Rust `TemporalField` and components.

use core::fmt;
use std::str::FromStr;

use crate::{
    components::{calendar::Calendar, Date, DateTime, MonthCode, PartialDate, YearMonthFields},
    error::TemporalError,
    TemporalResult,
};

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

impl From<TemporalFieldKey> for FieldMap {
    #[inline]
    fn from(value: TemporalFieldKey) -> Self {
        match value {
            TemporalFieldKey::Year => FieldMap::YEAR,
            TemporalFieldKey::Month => FieldMap::MONTH,
            TemporalFieldKey::MonthCode => FieldMap::MONTH_CODE,
            TemporalFieldKey::Day => FieldMap::DAY,
            TemporalFieldKey::Hour => FieldMap::HOUR,
            TemporalFieldKey::Minute => FieldMap::MINUTE,
            TemporalFieldKey::Second => FieldMap::SECOND,
            TemporalFieldKey::Millisecond => FieldMap::MILLISECOND,
            TemporalFieldKey::Microsecond => FieldMap::MICROSECOND,
            TemporalFieldKey::Nanosecond => FieldMap::NANOSECOND,
            TemporalFieldKey::Offset => FieldMap::OFFSET,
            TemporalFieldKey::Era => FieldMap::ERA,
            TemporalFieldKey::EraYear => FieldMap::ERA_YEAR,
            TemporalFieldKey::TimeZone => FieldMap::TIME_ZONE,
        }
    }
}

/// The post conversion field value.
#[derive(Debug, Clone)]
pub enum TemporalFieldValue {
    /// Designates the values as an integer.
    Integer(Option<i32>),
    /// Designates the value as a string.
    String(String),
}

impl From<i32> for TemporalFieldValue {
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
pub enum TemporalFieldKey {
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

impl TryFrom<FieldMap> for TemporalFieldKey {
    type Error = TemporalError;
    fn try_from(value: FieldMap) -> Result<Self, Self::Error> {
        match value {
            FieldMap::YEAR => Ok(TemporalFieldKey::Year),
            FieldMap::MONTH => Ok(TemporalFieldKey::Month),
            FieldMap::MONTH_CODE => Ok(TemporalFieldKey::MonthCode),
            FieldMap::DAY => Ok(TemporalFieldKey::Day),
            FieldMap::HOUR => Ok(TemporalFieldKey::Hour),
            FieldMap::MINUTE => Ok(TemporalFieldKey::Minute),
            FieldMap::SECOND => Ok(TemporalFieldKey::Second),
            FieldMap::MILLISECOND => Ok(TemporalFieldKey::Millisecond),
            FieldMap::MICROSECOND => Ok(TemporalFieldKey::Microsecond),
            FieldMap::NANOSECOND => Ok(TemporalFieldKey::Nanosecond),
            FieldMap::OFFSET => Ok(TemporalFieldKey::Offset),
            FieldMap::ERA => Ok(TemporalFieldKey::Era),
            FieldMap::ERA_YEAR => Ok(TemporalFieldKey::EraYear),
            FieldMap::TIME_ZONE => Ok(TemporalFieldKey::TimeZone),
            _ => Err(TemporalError::range().with_message("Invalid FieldMap bit value.")),
        }
    }
}

impl FromStr for TemporalFieldKey {
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
                "String cannot be converted to TemporalFieldKey",
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
    pub(crate) month_code: Option<MonthCode>,
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

    /// Sets a field as active. This will require the field's default value to be used if the field is not yet set.
    #[inline]
    pub fn activate_field(&mut self, key: TemporalFieldKey) {
        self.bit_map.set(key.into(), true);
    }

    /// Gets the value of a `TemporalFieldKey` if the field has been set to active. If the field
    /// has not been set, then return `None`.
    #[inline]
    pub fn get(&self, key: TemporalFieldKey) -> Option<TemporalFieldValue> {
        if !self.bit_map.contains(key.into()) {
            return None;
        }

        match key {
            TemporalFieldKey::Year => Some(TemporalFieldValue::Integer(self.year)),
            TemporalFieldKey::Month => Some(TemporalFieldValue::Integer(self.month)),
            TemporalFieldKey::MonthCode => Some(TemporalFieldValue::String(
                self.month_code
                    .map_or(String::default(), |s| s.as_str().to_owned()),
            )),
            TemporalFieldKey::Day => Some(TemporalFieldValue::Integer(self.day)),
            TemporalFieldKey::Hour => Some(TemporalFieldValue::from(self.hour)),
            TemporalFieldKey::Minute => Some(TemporalFieldValue::from(self.minute)),
            TemporalFieldKey::Second => Some(TemporalFieldValue::from(self.second)),
            TemporalFieldKey::Millisecond => Some(TemporalFieldValue::from(self.millisecond)),
            TemporalFieldKey::Microsecond => Some(TemporalFieldValue::from(self.microsecond)),
            TemporalFieldKey::Nanosecond => Some(TemporalFieldValue::from(self.nanosecond)),
            TemporalFieldKey::Offset => Some(TemporalFieldValue::String(
                self.offset.map_or(String::default(), |s| s.to_string()),
            )),
            TemporalFieldKey::Era => Some(TemporalFieldValue::String(
                self.era.map_or(String::default(), |s| s.to_string()),
            )),
            TemporalFieldKey::EraYear => Some(TemporalFieldValue::Integer(self.era_year)),
            TemporalFieldKey::TimeZone => Some(TemporalFieldValue::String(
                self.time_zone.map_or(String::default(), |s| s.to_string()),
            )),
        }
    }

    /// Validate and insert a key-value pair. This will also set the field as acitve if the value was successfully inserted.
    #[inline]
    pub fn insert(
        &mut self,
        key: TemporalFieldKey,
        value: TemporalFieldValue,
    ) -> TemporalResult<()> {
        match key {
            TemporalFieldKey::Year => {
                let TemporalFieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.year = value;
            }
            TemporalFieldKey::Month => {
                let TemporalFieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.month = value;
            }
            TemporalFieldKey::MonthCode => {
                let TemporalFieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.month_code = Some(MonthCode::from_str(&value)?);
            }
            TemporalFieldKey::Day => {
                let TemporalFieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.day = value;
            }
            TemporalFieldKey::Hour => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.hour = value;
            }
            TemporalFieldKey::Minute => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.minute = value;
            }
            TemporalFieldKey::Second => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.second = value;
            }
            TemporalFieldKey::Millisecond => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.millisecond = value;
            }
            TemporalFieldKey::Microsecond => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.microsecond = value;
            }
            TemporalFieldKey::Nanosecond => {
                let TemporalFieldValue::Integer(Some(value)) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.nanosecond = value;
            }
            TemporalFieldKey::Offset => {
                let TemporalFieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.offset = Some(
                    TinyAsciiStr::<16>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid offset string."))?,
                );
            }
            TemporalFieldKey::Era => {
                let TemporalFieldValue::String(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.era = Some(
                    TinyAsciiStr::<16>::from_str(&value)
                        .map_err(|_| TemporalError::general("Invalid era identifier."))?,
                );
            }
            TemporalFieldKey::EraYear => {
                let TemporalFieldValue::Integer(value) = value else {
                    return Err(
                        TemporalError::r#type().with_message("Invalid type for temporal field.")
                    );
                };
                self.day = value;
            }
            TemporalFieldKey::TimeZone => {
                let TemporalFieldValue::String(value) = value else {
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

        let month_code_int: i32 = (mc as u8).into();

        if self.month.is_some() && self.month != Some(month_code_int) {
            return Err(
                TemporalError::range().with_message("month and monthCode cannot be resolved.")
            );
        }

        self.insert(
            TemporalFieldKey::Month,
            TemporalFieldValue::from(month_code_int),
        )?;

        Ok(())
    }

    // TODO: Determine if this should be moved to `Calendar`.
    /// Merges two `TemporalFields` depending on the calendar.
    #[inline]
    pub fn merge_fields(&self, other: &Self, calendar: &Calendar) -> TemporalResult<Self> {
        let overridden_keys = calendar.field_keys_to_ignore(other.bit_map)?;

        let mut result = Self::default();

        for key in self.bit_map.iter() {
            let value = if overridden_keys.contains(key) {
                other.get(key.try_into()?)
            } else {
                self.get(key.try_into()?)
            };

            if let Some(value) = value {
                result.insert(key.try_into()?, value)?;
            };
        }

        Ok(result)
    }
}

impl From<&DateTime> for TemporalFields {
    fn from(value: &DateTime) -> Self {
        Self {
            bit_map: FieldMap::YEAR | FieldMap::MONTH | FieldMap::MONTH_CODE | FieldMap::DAY,
            year: Some(value.iso.date.year),
            month: Some(value.iso.date.month.into()),
            month_code: Some(
                MonthCode::try_from(value.iso.date.month)
                    .expect("Date must always have a valid month."),
            ),
            day: Some(value.iso.date.day.into()),
            ..Default::default()
        }
    }
}

impl From<&Date> for TemporalFields {
    fn from(value: &Date) -> Self {
        Self {
            bit_map: FieldMap::YEAR | FieldMap::MONTH | FieldMap::MONTH_CODE | FieldMap::DAY,
            year: Some(value.iso.year),
            month: Some(value.iso.month.into()),
            month_code: Some(
                MonthCode::try_from(value.iso.month).expect("Date must always have a valid month."),
            ),
            day: Some(value.iso.day.into()),
            ..Default::default()
        }
    }
}

impl From<PartialDate> for TemporalFields {
    fn from(value: PartialDate) -> Self {
        let mut bit_map = FieldMap::empty();
        if value.year.is_some() {
            bit_map.set(FieldMap::YEAR, true)
        };
        if value.month.is_some() {
            bit_map.set(FieldMap::MONTH, true)
        };
        if value.month_code.is_some() {
            bit_map.set(FieldMap::MONTH_CODE, true)
        };
        if value.day.is_some() {
            bit_map.set(FieldMap::DAY, true)
        };
        if value.era.is_some() {
            bit_map.set(FieldMap::ERA, true)
        }
        if value.era_year.is_some() {
            bit_map.set(FieldMap::ERA_YEAR, true)
        }

        Self {
            bit_map,
            year: value.year,
            month: value.month,
            month_code: value.month_code,
            day: value.day,
            era: value.era,
            era_year: value.era_year,
            ..Default::default()
        }
    }
}

// Conversion to `TemporalFields`
impl From<YearMonthFields> for TemporalFields {
    fn from(value: YearMonthFields) -> Self {
        TemporalFields {
            bit_map: FieldMap::YEAR | FieldMap::MONTH,
            year: Some(value.0),
            month: Some(value.1.into()),
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
    type Item = TemporalFieldKey;

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
    type Item = TemporalFieldValue;

    fn next(&mut self) -> Option<Self::Item> {
        let field = self.iter.next()?;

        match field {
            FieldMap::YEAR => Some(TemporalFieldValue::Integer(self.fields.year)),
            FieldMap::MONTH => Some(TemporalFieldValue::Integer(self.fields.month)),
            FieldMap::MONTH_CODE => Some(TemporalFieldValue::String(
                self.fields
                    .month_code
                    .map_or(String::default(), |s| s.as_str().to_owned()),
            )),
            FieldMap::DAY => Some(TemporalFieldValue::Integer(self.fields.day)),
            FieldMap::HOUR => Some(TemporalFieldValue::from(self.fields.hour)),
            FieldMap::MINUTE => Some(TemporalFieldValue::from(self.fields.minute)),
            FieldMap::SECOND => Some(TemporalFieldValue::from(self.fields.second)),
            FieldMap::MILLISECOND => Some(TemporalFieldValue::from(self.fields.millisecond)),
            FieldMap::MICROSECOND => Some(TemporalFieldValue::from(self.fields.microsecond)),
            FieldMap::NANOSECOND => Some(TemporalFieldValue::from(self.fields.nanosecond)),
            FieldMap::OFFSET => Some(TemporalFieldValue::String(
                self.fields
                    .offset
                    .map_or(String::default(), |s| s.to_string()),
            )),
            FieldMap::ERA => Some(TemporalFieldValue::String(
                self.fields.era.map_or(String::default(), |s| s.to_string()),
            )),
            FieldMap::ERA_YEAR => Some(TemporalFieldValue::Integer(self.fields.era_year)),
            FieldMap::TIME_ZONE => Some(TemporalFieldValue::String(
                self.fields
                    .time_zone
                    .map_or(String::default(), |s| s.to_string()),
            )),
            _ => None,
        }
    }
}
