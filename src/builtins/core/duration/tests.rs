use crate::{partial::PartialDuration, primitive::FiniteF64};

use super::Duration;

#[test]
fn partial_duration_empty() {
    let err = Duration::from_partial_duration(PartialDuration::default());
    assert!(err.is_err())
}

#[test]
fn partial_duration_values() {
    let mut partial = PartialDuration::default();
    let _ = partial.years.insert(FiniteF64(20.0));
    let result = Duration::from_partial_duration(partial).unwrap();
    assert_eq!(result.years(), 20.0);
}

