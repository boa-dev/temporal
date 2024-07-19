//! Implementation of a `DateDuration`

use crate::{utils::FiniteF64, Sign, TemporalError, TemporalResult};

/// `DateDuration` represents the [date duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-date-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct DateDuration {
    /// `DateDuration`'s internal year value.
    pub years: FiniteF64,
    /// `DateDuration`'s internal month value.
    pub months: FiniteF64,
    /// `DateDuration`'s internal week value.
    pub weeks: FiniteF64,
    /// `DateDuration`'s internal day value.
    pub days: FiniteF64,
}

impl DateDuration {
    /// Creates a new, non-validated `DateDuration`.
    #[inline]
    #[must_use]
    pub(crate) const fn new_unchecked(
        years: FiniteF64,
        months: FiniteF64,
        weeks: FiniteF64,
        days: FiniteF64,
    ) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }

    /// Returns the iterator for `DateDuration`
    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> Vec<FiniteF64> {
        Vec::from(&[self.years, self.months, self.weeks, self.days])
    }
}

impl DateDuration {
    /// Creates a new `DateDuration` with provided values.
    #[inline]
    pub fn new(
        years: FiniteF64,
        months: FiniteF64,
        weeks: FiniteF64,
        days: FiniteF64,
    ) -> TemporalResult<Self> {
        let result = Self::new_unchecked(years, months, weeks, days);
        if !super::is_valid_duration(
            years,
            months,
            weeks,
            days,
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
            FiniteF64::default(),
        ) {
            return Err(TemporalError::range().with_message("Invalid DateDuration."));
        }
        Ok(result)
    }

    /// Returns a negated `DateDuration`.
    #[inline]
    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            years: self.years.negate(),
            months: self.months.negate(),
            weeks: self.weeks.negate(),
            days: self.days.negate(),
        }
    }

    /// Returns a new `DateDuration` representing the absolute value of the current.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            years: self.years.abs(),
            months: self.months.abs(),
            weeks: self.weeks.abs(),
            days: self.days.abs(),
        }
    }

    /// Returns the sign for the current `DateDuration`.
    #[inline]
    #[must_use]
    pub fn sign(&self) -> Sign {
        super::duration_sign(&self.fields())
    }
}
