//! This module implements a native Rust `TemporalField` and components.

use std::{collections::hash_map::Keys, str::FromStr};

use crate::{components::calendar::Calendar, error::TemporalError, iso::IsoDate, TemporalResult};

use bitflags::bitflags;
use rustc_hash::FxHashMap;
// use rustc_hash::FxHashSet;

bitflags! {
    /// FieldMap maps the currently active fields on the `TemporalField`
    #[derive(Debug, PartialEq, Eq)]
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

/// The post conversion field value.
#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
pub enum FieldValue {
    /// Designates the values as an integer.
    Integer(i32),
    /// Designates the value as a string.
    String(String),
}

impl From<i32> for FieldValue {
    fn from(value: i32) -> Self {
        FieldValue::Integer(value)
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

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
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

impl FieldKey {
    fn default_value(&self) -> Option<FieldValue> {
        match self {
            Self::Year => None,
            Self::Month => None,
            Self::MonthCode => None,
            Self::Day => None,
            Self::Hour => Some(FieldValue::Integer(0)),
            Self::Minute => Some(FieldValue::Integer(0)),
            Self::Second => Some(FieldValue::Integer(0)),
            Self::Millisecond => Some(FieldValue::Integer(0)),
            Self::Microsecond => Some(FieldValue::Integer(0)),
            Self::Nanosecond => Some(FieldValue::Integer(0)),
            Self::Offset => None,
            Self::Era => None,
            Self::EraYear => None,
            Self::TimeZone => None,
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
    properties: FxHashMap<FieldKey, FieldValue>,
}

impl TemporalFields {
    pub fn keys(&self) -> Keys<'_, FieldKey, FieldValue> {
        self.properties.keys()
    }

    /// Get the value of the provided `FieldKey`
    pub fn get(&self, key: &FieldKey) -> Option<&FieldValue> {
        self.properties.get(key)
    }

    /// Sets the provided FieldKey value to it's default
    pub fn set_default(&mut self, key: FieldKey) {
        if let Some(value) = key.default_value() {
            let _ = self.properties.insert(key, value);
        }
    }

    /// Validate and insert a key-value pair.
    pub fn insert(
        &mut self,
        key: FieldKey,
        value: FieldValue,
    ) -> TemporalResult<Option<FieldValue>> {
        match key {
            FieldKey::Year
            | FieldKey::Month
            | FieldKey::Day
            | FieldKey::Hour
            | FieldKey::Minute
            | FieldKey::Second
            | FieldKey::Millisecond
            | FieldKey::Microsecond
            | FieldKey::Nanosecond
            | FieldKey::EraYear => {
                if !matches!(value, FieldValue::Integer(_)) {
                    return Err(TemporalError::r#type().with_message("Invalid Field type."));
                }
                Ok(self.properties.insert(key, value))
            }
            FieldKey::MonthCode | FieldKey::Offset | FieldKey::TimeZone | FieldKey::Era => {
                if !matches!(value, FieldValue::String(_)) {
                    return Err(TemporalError::r#type().with_message("Invalid Field type."));
                }
                Ok(self.properties.insert(key, value))
            }
        }
    }

    /// Resolve `TemporalFields` month and monthCode fields.
    pub(crate) fn iso_resolve_month(&mut self) -> TemporalResult<()> {
        let Some(mc) = self.properties.get(&FieldKey::MonthCode) else {
            let result = match self.properties.get(&FieldKey::Month) {
                Some(_) => Ok(()),
                None => Err(TemporalError::range()
                    .with_message("month and MonthCode values cannot both be undefined.")),
            };

            return result;
        };

        let FieldValue::String(unresolved_month_code) = mc else {
            return Err(TemporalError::assert());
        };

        // MonthCode is present and needs to be resolved.

        let month_code_integer = month_code_to_integer(unresolved_month_code)?;

        let new_month = match self.properties.get(&FieldKey::Month) {
            Some(&FieldValue::Integer(month)) if month != month_code_integer => {
                return Err(
                    TemporalError::range().with_message("month and monthCode cannot be resolved.")
                )
            }
            _ => month_code_integer,
        };

        let _ = self
            .properties
            .insert(FieldKey::Month, FieldValue::Integer(new_month));

        Ok(())
    }

    // TODO: Determine if this should be moved to `Calendar`.
    /// Merges two `TemporalFields` depending on the calendar.
    pub fn merge_fields(&self, other: &Self, calendar: Calendar) -> TemporalResult<Self> {
        let add_keys = other.keys().copied().collect::<Vec<_>>();
        let overridden_keys = calendar.field_keys_to_ignore(&add_keys)?;

        let mut result = Self::default();

        for key in self.keys() {
            let value = if overridden_keys.contains(key) {
                other.get(key)
            } else {
                self.get(key)
            };

            if let Some(value) = value {
                result.insert(*key, value.clone())?;
            }
        }

        Ok(result)
    }
}

impl TemporalFields {
    pub fn year(&self) -> Option<i32> {
        let Some(FieldValue::Integer(i)) = self.get(&FieldKey::Year) else {
            return None;
        };
        Some(*i)
    }

    pub fn month(&self) -> Option<i32> {
        let Some(FieldValue::Integer(i)) = self.get(&FieldKey::Month) else {
            return None;
        };
        Some(*i)
    }

    pub fn month_code(&self) -> String {
        let Some(FieldValue::String(mc)) = self.get(&FieldKey::MonthCode) else {
            return String::default();
        };
        mc.clone()
    }

    pub fn day(&self) -> Option<i32> {
        let Some(FieldValue::Integer(i)) = self.get(&FieldKey::Day) else {
            return None;
        };
        Some(*i)
    }

    pub fn era(&self) -> String {
        let Some(FieldValue::String(era)) = self.get(&FieldKey::Era) else {
            return String::default();
        };
        era.clone()
    }
}

impl From<IsoDate> for TemporalFields {
    fn from(value: IsoDate) -> Self {
        let mut fields = Self::default();
        let _ = fields.insert(FieldKey::Year, FieldValue::Integer(value.year));
        let _ = fields.insert(FieldKey::Month, FieldValue::Integer(value.month.into()));
        let _ = fields.insert(FieldKey::Day, FieldValue::Integer(value.day.into()));
        fields
    }
}

fn month_code_to_integer(mc: &str) -> TemporalResult<i32> {
    match mc {
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
