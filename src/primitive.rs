//! Implementation of the FiniteF64 primitive

use crate::{TemporalError, TemporalResult};
use num_traits::{AsPrimitive, FromPrimitive, PrimInt};

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

    /// Returns an integer of type `T` if if value is integral
    pub fn as_integer_if_integral<T: PrimInt + AsPrimitive<f64>>(&self) -> TemporalResult<T>
    where
        f64: AsPrimitive<T>,
    {
        if self.0 != self.0.trunc() {
            return Err(TemporalError::range().with_message("value must be integral."));
        }
        Ok(self.0.as_())
    }

    /// Truncate the current `FiniteF64` to an integer `T`
    pub fn truncate<T: PrimInt + AsPrimitive<f64>>(&self) -> T
    where
        f64: AsPrimitive<T>,
    {
        let clamped =
            num_traits::clamp(self.as_inner(), T::min_value().as_(), T::max_value().as_());
        clamped.as_()
    }

    /// Truncate the current `FiniteF64` to an integer `T`, throwing an error if the value is not positive.
    pub fn truncate_as_positive<T: PrimInt + AsPrimitive<f64>>(&self) -> TemporalResult<T>
    where
        f64: AsPrimitive<T>,
        i8: AsPrimitive<T>,
    {
        let truncated_value = self.truncate::<T>();
        if truncated_value <= 0i8.as_() {
            return Err(TemporalError::range().with_message("integer must be positive."));
        }
        Ok(truncated_value)
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

impl TryFrom<i64> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let result = f64::from_i64(value)
            .ok_or(TemporalError::range().with_message("number exceeded a valid range."))?;
        Ok(Self(result))
    }
}

impl TryFrom<u64> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let result = f64::from_u64(value)
            .ok_or(TemporalError::range().with_message("number exceeded a valid range."))?;
        Ok(Self(result))
    }
}

impl TryFrom<i128> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let result = f64::from_i128(value)
            .ok_or(TemporalError::range().with_message("number exceeded a valid range."))?;
        if !result.is_finite() {
            return Err(TemporalError::range().with_message("number value is not a finite value."));
        }
        Ok(Self(result))
    }
}

impl TryFrom<u128> for FiniteF64 {
    type Error = TemporalError;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let result = f64::from_u128(value)
            .ok_or(TemporalError::range().with_message("number exceeded a valid range."))?;
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

impl From<i16> for FiniteF64 {
    fn from(value: i16) -> Self {
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
    fn partial_cmp(&self, other: &f64) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

#[cfg(test)]
mod tests {
    use super::FiniteF64;

    #[test]
    fn finitef64_truncate() {
        let value = 8_640_000_000_000_000i64;
        let finite = FiniteF64::try_from(value).unwrap();

        let num_usize = finite.truncate::<usize>();
        #[cfg(target_pointer_width = "64")]
        assert_eq!(num_usize, 8_640_000_000_000_000);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(num_usize, usize::MAX);
        let num_u8 = finite.truncate::<u8>();
        assert_eq!(num_u8, u8::MAX);
        let num_u16 = finite.truncate::<u16>();
        assert_eq!(num_u16, u16::MAX);
        let num_u32 = finite.truncate::<u32>();
        assert_eq!(num_u32, u32::MAX);
        let num_u64 = finite.truncate::<u64>();
        assert_eq!(num_u64, 8_640_000_000_000_000);
        let num_u128 = finite.truncate::<u128>();
        assert_eq!(num_u128, 8_640_000_000_000_000);

        let num_isize = finite.truncate::<isize>();
        #[cfg(target_pointer_width = "64")]
        assert_eq!(num_isize, 8_640_000_000_000_000);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(num_isize, isize::MAX);
        let num_i8 = finite.truncate::<i8>();
        assert_eq!(num_i8, i8::MAX);
        let num_i16 = finite.truncate::<i16>();
        assert_eq!(num_i16, i16::MAX);
        let num_i32 = finite.truncate::<i32>();
        assert_eq!(num_i32, i32::MAX);
        let num_i64 = finite.truncate::<i64>();
        assert_eq!(num_i64, 8_640_000_000_000_000);
        let num_i128 = finite.truncate::<i128>();
        assert_eq!(num_i128, 8_640_000_000_000_000);
    }

    #[test]
    fn finitef64_truncate_as_positive_int() {
        let positive = FiniteF64::from(5);
        let truncated = positive.truncate_as_positive::<i32>().unwrap();
        assert_eq!(truncated, 5);

        let negative = FiniteF64::from(-4);
        let truncated = negative.truncate_as_positive::<i32>();
        assert!(truncated.is_err());
    }

    #[test]
    fn finitef64_truncate_as_positive_correct_floor() {
        let floors_to_zero = FiniteF64::try_from(0.5).unwrap();
        let truncated = floors_to_zero.truncate_as_positive::<i32>();
        assert!(truncated.is_err());

        let floors_to_zero = FiniteF64::try_from(-0.5).unwrap();
        let truncated = floors_to_zero.truncate_as_positive::<i32>();
        assert!(truncated.is_err());

        let floors_to_zero = FiniteF64::try_from(0.9).unwrap();
        let truncated = floors_to_zero.truncate_as_positive::<i32>();
        assert!(truncated.is_err());

        let floors_to_one = FiniteF64::try_from(1.1).unwrap();
        let truncated = floors_to_one.truncate_as_positive::<i32>().unwrap();
        assert_eq!(truncated, 1);

        let floors_to_one = FiniteF64::try_from(1.9).unwrap();
        let truncated = floors_to_one.truncate_as_positive::<i32>().unwrap();
        assert_eq!(truncated, 1);
    }

    #[test]
    fn finitef64_truncate_correct_floor() {
        let floors_to_zero = FiniteF64::try_from(0.5).unwrap();
        let truncated = floors_to_zero.truncate::<i32>();
        assert_eq!(truncated, 0);

        let floors_to_zero = FiniteF64::try_from(-0.5).unwrap();
        let truncated = floors_to_zero.truncate::<i32>();
        assert_eq!(truncated, 0);

        let floors_to_one = FiniteF64::try_from(-1.2).unwrap();
        let truncated = floors_to_one.truncate::<i32>();
        assert_eq!(truncated, -1);

        let floors_to_one = FiniteF64::try_from(1.9).unwrap();
        let truncated = floors_to_one.truncate::<i32>();
        assert_eq!(truncated, 1);
    }

    #[test]
    fn finitef64_integer_if_integral_returns_err() {
        let test_value = FiniteF64::try_from(0.999).unwrap();
        let integer = test_value.as_integer_if_integral::<u8>();
        assert!(integer.is_err());

        let test_value = FiniteF64::try_from(1.5).unwrap();
        let integer = test_value.as_integer_if_integral::<u8>();
        assert!(integer.is_err());
    }

    #[test]
    fn finitef64_integer_if_integral_returns_int() {
        let test_value = FiniteF64::try_from(0).unwrap();
        let integer = test_value.as_integer_if_integral::<u8>();
        assert_eq!(integer, Ok(0));

        let test_value = FiniteF64::try_from(1).unwrap();
        let integer = test_value.as_integer_if_integral::<u8>();
        assert_eq!(integer, Ok(1));
    }
}
