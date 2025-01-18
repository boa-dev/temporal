use crate::TemporalError;
use crate::{builtins::core, TemporalResult};
use crate::builtins::std::timezone::TZ_PROVIDER;

use super::{date::PlainDate, ZonedDateTime};

use core::options::RelativeTo as CoreRelativeTo;

#[derive(Debug, Clone)]
pub enum RelativeTo {
    PlainDate(PlainDate),
    ZonedDateTime(ZonedDateTime),
}

impl From<PlainDate> for RelativeTo {
    fn from(value: PlainDate) -> Self {
        Self::PlainDate(value)
    }
}

impl From<ZonedDateTime> for RelativeTo {
    fn from(value: ZonedDateTime) -> Self {
        Self::ZonedDateTime(value)
    }
}

impl From<CoreRelativeTo> for RelativeTo {
    fn from(value: CoreRelativeTo) -> Self {
        match value {
            CoreRelativeTo::PlainDate(d) => Self::PlainDate(d.into()),
            CoreRelativeTo::ZonedDateTime(d) => Self::ZonedDateTime(d.into()),
        }
    }
}

impl From<RelativeTo> for CoreRelativeTo {
    fn from(value: RelativeTo) -> Self {
        match value {
            RelativeTo::PlainDate(d) => CoreRelativeTo::PlainDate(d.0),
            RelativeTo::ZonedDateTime(zdt) => CoreRelativeTo::ZonedDateTime(zdt.0),
        }
    }
}

impl RelativeTo {
    pub fn try_from_str(source: &str) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        core::options::RelativeTo::try_from_str_with_provider(source, &*provider).map(Into::into)
    }
}

