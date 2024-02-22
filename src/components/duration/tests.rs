use crate::{
    components::{calendar::CalendarSlot, Date},
    options::ArithmeticOverflow,
};

use super::*;

// roundingmode-floor.js
#[test]
fn basic_floor_rounding() {
    const UNITS: [&str; 10] = [
        "years",
        "months",
        "weeks",
        "days",
        "hours",
        "minutes",
        "seconds",
        "milliseconds",
        "microseconds",
        "nanoseconds",
    ];
    const EXPECTED_POSITIVE: [[i32; 10]; 10] = [
        [5, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 6, 8, 0, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 987, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 987, 500],
    ];
    const EXPECTED_NEG: [[i32; 10]; 10] = [
        [-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -6, -9, 0, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],
    ];

    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let mode = TemporalRoundingMode::Floor;
    let forward_date = Date::<()>::new(
        2020,
        4,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();
    let backward_date = Date::<()>::new(
        2020,
        12,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };
    let relative_backward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    for i in 0..10 {
        let unit = TemporalUnit::from_str(UNITS[i]).unwrap();
        let result = test_duration
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_forward,
                &mut (),
            )
            .unwrap();
        assert!(result
            .iter()
            .zip(&EXPECTED_POSITIVE[i])
            .all(|r| r.0 as i32 == *r.1));
        let neg_result = test_duration
            .negated()
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_backward,
                &mut (),
            )
            .unwrap();
        assert!(neg_result
            .iter()
            .zip(&EXPECTED_NEG[i])
            .all(|r| r.0 as i32 == *r.1));
    }
}

// roundingmode-ceil.js
#[test]
fn basic_ceil_rounding() {
    const UNITS: [&str; 10] = [
        "years",
        "months",
        "weeks",
        "days",
        "hours",
        "minutes",
        "seconds",
        "milliseconds",
        "microseconds",
        "nanoseconds",
    ];
    const EXPECTED_POSITIVE: [[i32; 10]; 10] = [
        [6, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 6, 9, 0, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 28, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 17, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 31, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 21, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 124, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 988, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 987, 500],
    ];
    const EXPECTED_NEG: [[i32; 10]; 10] = [
        [-5, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -6, -8, 0, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -987, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],
    ];

    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let mode = TemporalRoundingMode::Ceil;
    let forward_date = Date::<()>::new(
        2020,
        4,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();
    let backward_date = Date::<()>::new(
        2020,
        12,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };
    let relative_backward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    for i in 0..10 {
        let unit = TemporalUnit::from_str(UNITS[i]).unwrap();
        let result = test_duration
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_forward,
                &mut (),
            )
            .unwrap();
        assert!(result
            .iter()
            .zip(&EXPECTED_POSITIVE[i])
            .all(|r| r.0 as i32 == *r.1));
        let neg_result = test_duration
            .negated()
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_backward,
                &mut (),
            )
            .unwrap();
        assert!(neg_result
            .iter()
            .zip(&EXPECTED_NEG[i])
            .all(|r| r.0 as i32 == *r.1));
    }
}

// roundingmode-expand.js
#[test]
fn basic_expand_rounding() {
    const UNITS: [&str; 10] = [
        "years",
        "months",
        "weeks",
        "days",
        "hours",
        "minutes",
        "seconds",
        "milliseconds",
        "microseconds",
        "nanoseconds",
    ];
    const EXPECTED_POSITIVE: [[i32; 10]; 10] = [
        [6, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 6, 9, 0, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 28, 0, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 17, 0, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 31, 0, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 21, 0, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 124, 0, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 988, 0],
        [5, 7, 0, 27, 16, 30, 20, 123, 987, 500],
    ];
    const EXPECTED_NEG: [[i32; 10]; 10] = [
        [-6, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -8, 0, 0, 0, 0, 0, 0, 0, 0],
        [-5, -6, -9, 0, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -28, 0, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -17, 0, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -31, 0, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -21, 0, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -124, 0, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -988, 0],
        [-5, -7, 0, -27, -16, -30, -20, -123, -987, -500],
    ];

    let test_duration =
        Duration::new(5.0, 6.0, 7.0, 8.0, 40.0, 30.0, 20.0, 123.0, 987.0, 500.0).unwrap();
    let mode = TemporalRoundingMode::Expand;
    let forward_date = Date::<()>::new(
        2020,
        4,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();
    let backward_date = Date::<()>::new(
        2020,
        12,
        1,
        CalendarSlot::from_str("iso8601").unwrap(),
        ArithmeticOverflow::Reject,
    )
    .unwrap();

    let relative_forward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&forward_date),
        zdt: None,
    };
    let relative_backward: RelativeTo<'_, (), ()> = RelativeTo {
        date: Some(&backward_date),
        zdt: None,
    };

    for i in 0..10 {
        let unit = TemporalUnit::from_str(UNITS[i]).unwrap();
        let result = test_duration
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_forward,
                &mut (),
            )
            .unwrap();
        assert!(result
            .iter()
            .zip(&EXPECTED_POSITIVE[i])
            .all(|r| r.0 as i32 == *r.1));
        let neg_result = test_duration
            .negated()
            .round(
                None,
                Some(unit),
                None,
                Some(mode),
                &relative_backward,
                &mut (),
            )
            .unwrap();
        assert!(neg_result
            .iter()
            .zip(&EXPECTED_NEG[i])
            .all(|r| r.0 as i32 == *r.1));
    }
}
