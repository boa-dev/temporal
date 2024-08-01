//! Implementation of the FiniteF64 primitive

use crate::{TemporalError, TemporalResult};
use num_traits::{AsPrimitive, FromPrimitive};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteF64(pub(crate) f64);

impl FiniteF64 {
    #[inline]
    pub fn as_inner(&self) -> f64 {
        self.0
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    #[inline]
    pub fn negate(&self) -> Self {
        if !self.is_zero() {
            Self(self.0 * -1.0)
        } else {
            *self
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn checked_add(&self, other: &Self) -> TemporalResult<Self> {
        let result = Self(self.0 + other.0);
        if !result.0.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(result)
    }

    #[inline]
    pub fn checked_mul_add(&self, a: FiniteF64, b: FiniteF64) -> TemporalResult<Self> {
        let result = Self(self.0.mul_add(a.0, b.0));
        if !result.0.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(result)
    }

    pub fn copysign(&self, other: f64) -> Self {
        Self(self.0.copysign(other))
    }

    pub(crate) fn as_date_value(&self) -> TemporalResult<i32> {
        if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&self.0) {
            return Err(TemporalError::range().with_message("number exceeds a valid date value."));
        }
        Ok(self.0 as i32)
    }
}

impl AsPrimitive<i64> for FiniteF64 {
    fn as_(self) -> i64 {
        self.0 as i64
    }
}

impl AsPrimitive<i128> for FiniteF64 {
    fn as_(self) -> i128 {
        self.0 as i128
    }
}

impl TryFrom<f64> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(Self(value))
    }
}

impl TryFrom<i128> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let result = f64::from_i128(value)
            .ok_or(TemporalError::range().with_message("days exceeded a valid range."))?;
        if !result.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(Self(result))
    }
}

impl From<i8> for FiniteF64 {
    fn from(value: i8) -> Self {
        Self(f64::from(value))
    }
}

impl From<i32> for FiniteF64 {
    fn from(value: i32) -> Self {
        Self(f64::from(value))
    }
}

impl From<u8> for FiniteF64 {
    fn from(value: u8) -> Self {
        Self(f64::from(value))
    }
}

impl From<u16> for FiniteF64 {
    fn from(value: u16) -> Self {
        Self(f64::from(value))
    }
}

impl From<u32> for FiniteF64 {
    fn from(value: u32) -> Self {
        Self(f64::from(value))
    }
}

impl PartialEq<f64> for FiniteF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<f64> for FiniteF64 {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl From<FiniteF64> for f64 {
    fn from(value: FiniteF64) -> Self {
        value.0
    }
}
