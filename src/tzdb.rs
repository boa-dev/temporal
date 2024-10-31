// Relevant operations:
//
//  - Time Zone Identifiers
//  - AvailableNamedTimeZoneIdentifiers
//  - SystemTimeZoneIdentifier
//  - IsTimeZoneOffsetString
//  - GetNamedTimeZoneEpochNanoseconds
//     - fn(id, isoDateTimeRecord) -> [epochNanoseconds]
//  - GetNamedTimeZoneOffsetNanoseconds
//     - fn(id, epochNanoseconds) -> [offset]

// TODO: Potentially implement a IsoDateTimeRecord type to decouple
// public facing APIs from IsoDateTime

// Could return type be something like [Option<i128>; 2]

// NOTE: tzif data is computed in glibc's `__tzfile_compute` in `tzfile.c`.
//
// Handling the logic here may be incredibly important for full tzif support.

// NOTES:
//
// Transitions to DST (in march) + 1. Empty list between 2:00-3:00.
// Transitions to Std (in nov) -1. Two elements 1:00-2:00 is repeated twice.

// Transition Seconds + (offset diff)
// where
// offset diff = is_dst { dst_off - std_off } else { std_off - dst_off }, i.e. to_offset - from_offset

use std::path::{Path, PathBuf};

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use core::cell::RefCell;

use tzif::{
    self,
    data::{
        posix::PosixTzString,
        time::Seconds,
        tzif::{DataBlock, LocalTimeTypeRecord, TzifData, TzifHeader},
    },
};

use crate::{components::tz::TzProvider, iso::IsoDateTime, TemporalError, TemporalResult};

const ZONEINFO_DIR: &str = "/usr/share/zoneinfo/";

pub type TransitionInfo = [Option<LocalTimeTypeRecord>; 2];

#[derive(Debug, Clone)]
pub struct Tzif {
    pub header1: TzifHeader,
    pub data_block1: DataBlock,
    pub header2: Option<TzifHeader>,
    pub data_block2: Option<DataBlock>,
    pub footer: Option<PosixTzString>,
}

impl From<TzifData> for Tzif {
    fn from(value: TzifData) -> Self {
        let TzifData {
            header1,
            data_block1,
            header2,
            data_block2,
            footer,
        } = value;

        Self {
            header1,
            data_block1,
            header2,
            data_block2,
            footer,
        }
    }
}

impl Tzif {
    fn read_tzif(identifier: &str) -> TemporalResult<Self> {
        let mut path = PathBuf::from(ZONEINFO_DIR);
        path.push(identifier);
        Self::from_path(&path)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> TemporalResult<Self> {
        tzif::parse_tzif_file(path)
            .map(Into::into)
            .map_err(|e| TemporalError::general(e.to_string()))
    }

    pub fn get_data_block2(&self) -> TemporalResult<&DataBlock> {
        self.data_block2
            .as_ref()
            .ok_or(TemporalError::general("Only Tzif V2+ is supported."))
    }

    pub fn get(&self, epoch_seconds: &Seconds) -> TemporalResult<LocalTimeTypeRecord> {
        let db = self.get_data_block2()?;
        let result = db.transition_times.binary_search(epoch_seconds);

        match result {
            Ok(idx) => Ok(get_local_record(db, idx - 1)),
            Err(idx) if idx == 0 => Ok(get_local_record(db, idx)),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    return Err(TemporalError::general("TODO: Support POSIX tz string."));
                }
                Ok(get_local_record(db, idx - 1))
            }
        }
    }

    // For more information, see /docs/TZDB.md
    /// This function determines the Time Zone output for a local epoch
    /// nanoseconds value without an offset.
    ///
    /// Basically, if someone provides a DateTime 2017-11-05T01:30:00,
    /// we have no way of knowing if this value is in DST or STD.
    /// Furthermore, for the above example, this should return 2 time
    /// zones due to there being two 2017-11-05T01:30:00. On the other
    /// side of the transition, the DateTime 2017-03-12T02:30:00 could
    /// be provided. This time does NOT exist due to the +1 jump from
    /// 02:00 -> 03:00 (but of course it does as a nanosecond value).
    pub fn v2_estimate_tz_pair(
        &self,
        seconds: &Seconds,
    ) -> TemporalResult<Vec<LocalTimeTypeRecord>> {
        // We need to estimate a tz pair.
        // First search the ambiguous seconds.
        // TODO: it would be nice to resolve the Posix str into a local time type record.
        let db = self.get_data_block2()?;
        let b_search_result = db.transition_times.binary_search(seconds);

        let estimated_idx = match b_search_result {
            // TODO: Double check returning early here with tests.
            Ok(idx) => return Ok(vec![get_local_record(db, idx)]),
            Err(idx) if idx == 0 => return Ok(vec![get_local_record(db, idx)]),
            Err(idx) => {
                if db.transition_times.len() <= idx {
                    return Err(TemporalError::general("TODO: Support POSIX tz string."));
                }
                idx
            }
        };

        // The estimated index will be off based on the amount missing
        // from the lack of offset.
        //
        // This means that we may need (idx, idx - 1) or (idx - 1, idx - 2)
        let record = get_local_record(db, estimated_idx);
        let record_minus_one = get_local_record(db, estimated_idx - 1);

        // Q: Potential shift bugs with odd historical transitions? This
        //
        // Shifts the 2 rule window for positive zones that would have returned
        // a different idx.
        let shift_window = usize::from((record.utoff + record_minus_one.utoff) >= Seconds(0));

        let new_idx = estimated_idx - shift_window;

        let current_transition = db.transition_times[new_idx];
        let current_diff = *seconds - current_transition;

        let initial_record = get_local_record(db, new_idx - 1);
        let next_record = get_local_record(db, new_idx);

        // Adjust for offset inversion from northern/southern hemisphere.
        let offset_range = if initial_record.utoff < next_record.utoff {
            initial_record.utoff..next_record.utoff
        } else {
            next_record.utoff..initial_record.utoff
        };
        match offset_range.contains(&current_diff) {
            true if next_record.is_dst => Ok(Vec::default()),
            true => Ok(vec![next_record, initial_record]),
            false => Ok(vec![initial_record]),
        }
    }
}

#[inline]
fn get_local_record(db: &DataBlock, idx: usize) -> LocalTimeTypeRecord {
    db.local_time_type_records[db.transition_types[idx]]
}

#[derive(Debug, Default)]
pub struct FsTzdbProvider {
    cache: RefCell<BTreeMap<String, Tzif>>,
}

impl FsTzdbProvider {
    pub fn get(&self, identifier: &str) -> TemporalResult<Tzif> {
        if let Some(tzif) = self.cache.borrow().get(identifier) {
            return Ok(tzif.clone());
        }
        let tzif = Tzif::read_tzif(identifier)?;
        Ok(self
            .cache
            .borrow_mut()
            .entry(identifier.into())
            .or_insert(tzif)
            .clone())
    }
}

impl TzProvider for FsTzdbProvider {
    fn check_identifier(&self, identifier: &str) -> bool {
        self.get(identifier).is_ok()
    }

    fn get_named_tz_epoch_nanoseconds(
        &self,
        identifier: &str,
        iso_datetime: IsoDateTime,
    ) -> TemporalResult<Vec<i128>> {
        let seconds = (iso_datetime
            .as_nanoseconds()
            .expect("IsoDateTime to be valid")
            / 1_000_000_000) as i64;
        let tzif = self.get(identifier)?;
        let local_time_record_result = tzif.v2_estimate_tz_pair(&Seconds(seconds))?;
        Ok(local_time_record_result
            .iter()
            .map(|r| r.utoff.0 as i128 * 1_000_000_000)
            .collect())
    }

    fn get_named_tz_offset_nanoseconds(
        &self,
        identifier: &str,
        epoch_nanoseconds: i128,
    ) -> TemporalResult<i128> {
        let tzif = self.get(identifier)?;
        let seconds = (epoch_nanoseconds / 1_000_000_000) as i64;
        let local_time_record_result = tzif.get(&Seconds(seconds))?;
        Ok(local_time_record_result.utoff.0 as i128 * 1_000_000_000)
    }
}
//

#[cfg(test)]
mod tests {
    use tzif::data::time::Seconds;

    use crate::{iso::IsoDateTime, tzdb::TzProvider};

    use super::{FsTzdbProvider, Tzif};

    #[test]
    fn one_second_after_empty_edge_case() {
        let provider = FsTzdbProvider::default();
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 3,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();

        let local = provider
            .get_named_tz_epoch_nanoseconds("America/New_York", today)
            .unwrap();
        assert_eq!(local.len(), 1);
    }

    #[test]
    fn one_second_before_empty_edge_case() {
        let provider = FsTzdbProvider::default();
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 59,
            second: 59,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();

        let local = provider
            .get_named_tz_epoch_nanoseconds("America/New_York", today)
            .unwrap();
        assert!(local.is_empty());
    }

    #[test]
    fn new_york_empty_test_case() {
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 3,
            day: 12,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();
        assert!(locals.is_empty());
    }

    #[test]
    fn sydney_empty_test_case() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 10,
            day: 1,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert!(locals.is_empty());
    }

    #[test]
    fn new_york_duplicate_case() {
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 11,
            day: 5,
        };
        let time = crate::iso::IsoTime {
            hour: 1,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();

        assert_eq!(locals.len(), 2);
    }

    #[test]
    fn sydney_duplicate_case() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 2017,
            month: 4,
            day: 2,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert_eq!(locals.len(), 2);
    }

    // TODO: Determine the validity of this test. Primarily, this test
    // goes beyond the regularly historic limit of transition_times, so
    // even when on a DST boundary the first time zone is returned. The
    // question is whether this behavior is consistent with what would
    // be expected.
    #[test]
    fn before_epoch_northern_hemisphere() {
        let date = crate::iso::IsoDate {
            year: 1880,
            month: 11,
            day: 5,
        };
        let time = crate::iso::IsoTime {
            hour: 1,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let edge_case = IsoDateTime::new(date, time).unwrap();
        let edge_case_seconds = edge_case
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let new_york = Tzif::read_tzif("America/New_York");
        assert!(new_york.is_ok());
        let new_york = new_york.unwrap();

        let locals = new_york
            .v2_estimate_tz_pair(&Seconds(edge_case_seconds))
            .unwrap();

        assert_eq!(locals.len(), 1);
    }

    // TODO: Determine the validity of this test. Primarily, this test
    // goes beyond the regularly historic limit of transition_times, so
    // even when on a DST boundary the first time zone is returned. The
    // question is whether this behavior is consistent with what would
    // be expected.
    #[test]
    fn before_epoch_southern_hemisphere() {
        // Australia Daylight savings day
        let date = crate::iso::IsoDate {
            year: 1880,
            month: 4,
            day: 2,
        };
        let time = crate::iso::IsoTime {
            hour: 2,
            minute: 30,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        };
        let today = IsoDateTime::new(date, time).unwrap();
        let seconds = today
            .as_nanoseconds()
            .map_or(0, |nanos| (nanos / 1_000_000_000) as i64);

        let sydney = Tzif::read_tzif("Australia/Sydney");
        assert!(sydney.is_ok());
        let sydney = sydney.unwrap();

        let locals = sydney.v2_estimate_tz_pair(&Seconds(seconds)).unwrap();
        assert_eq!(locals.len(), 1);
    }
}
