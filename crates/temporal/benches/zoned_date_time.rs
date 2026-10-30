//! Benchmarks for `ZonedDateTime` field access.
//!
//! Mirrors the harness in <https://github.com/boa-dev/temporal/issues/756>: every
//! field getter resolves the ISO date-time from the epoch nanoseconds, so reading
//! N fields costs N conversions. The `PlainDateTime` rows are the floor those
//! getters should approach; the construction and arithmetic rows guard against
//! moving that cost onto paths that never read a field.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use temporal::options::{Disambiguation, OffsetDisambiguation};
use temporal::TimeZone;
use temporal::{Duration, PlainDateTime, ZonedDateTime};

/// An IXDTF string carrying both an offset and an IANA annotation, which is the
/// shape an engine actually parses.
const SOURCE: &str = "2025-01-01T00:00:00-05:00[America/New_York]";

fn parse(source: &str) -> ZonedDateTime {
    ZonedDateTime::from_utf8(
        source.as_bytes(),
        Disambiguation::Compatible,
        OffsetDisambiguation::Reject,
    )
    .expect("source must parse")
}

/// The step between fixtures: an uneven duration that crosses DST boundaries as
/// the series walks forward, so the spread is not all one offset.
fn step() -> Duration {
    Duration::new(0, 0, 0, 11, 7, 23, 41, 0, 0, 0).expect("valid duration")
}

/// A spread of zoned date-times, parsed once and then walked forward by adding a
/// duration, so the benchmark is not dominated by a single offset or code path.
fn zoned_date_times() -> Vec<ZonedDateTime> {
    let step = step();
    let mut current = parse(SOURCE);
    let mut out = Vec::with_capacity(64);
    for _ in 0..64 {
        out.push(current.clone());
        current = current
            .add(&step, None)
            .expect("addition must stay in range");
    }
    out
}

/// Reading seven fields — the shape a formatter or `toString` takes.
fn seven_getters(c: &mut Criterion) {
    let zdts = zoned_date_times();
    let pdts: Vec<PlainDateTime> = zdts.iter().map(|z| z.to_plain_date_time()).collect();

    let mut group = c.benchmark_group("seven_getters");
    group.bench_function("zoned_date_time", |b| {
        b.iter(|| {
            for zdt in &zdts {
                let zdt = black_box(zdt);
                black_box((
                    zdt.year(),
                    zdt.month(),
                    zdt.day(),
                    zdt.hour(),
                    zdt.minute(),
                    zdt.second(),
                    zdt.millisecond(),
                ));
            }
        })
    });
    group.bench_function("plain_date_time", |b| {
        b.iter(|| {
            for pdt in &pdts {
                let pdt = black_box(pdt);
                black_box((
                    pdt.year(),
                    pdt.month(),
                    pdt.day(),
                    pdt.hour(),
                    pdt.minute(),
                    pdt.second(),
                    pdt.millisecond(),
                ));
            }
        })
    });
    group.bench_function("zoned_via_plain_date_time", |b| {
        b.iter(|| {
            for zdt in &zdts {
                let pdt = black_box(zdt).to_plain_date_time();
                black_box((
                    pdt.year(),
                    pdt.month(),
                    pdt.day(),
                    pdt.hour(),
                    pdt.minute(),
                    pdt.second(),
                    pdt.millisecond(),
                ));
            }
        })
    });
    group.finish();
}

/// A single getter, separating the time fields (pure ISO arithmetic) from the
/// calendar fields (which additionally build an ICU4X date).
fn single_getter(c: &mut Criterion) {
    let zdts = zoned_date_times();

    let mut group = c.benchmark_group("single_getter");
    group.bench_function("hour", |b| {
        b.iter(|| {
            for zdt in &zdts {
                black_box(black_box(zdt).hour());
            }
        })
    });
    group.bench_function("year", |b| {
        b.iter(|| {
            for zdt in &zdts {
                black_box(black_box(zdt).year());
            }
        })
    });
    group.finish();
}

/// Construction, which an eagerly-computed cache makes more expensive. Parsing is
/// how an engine reaches `ZonedDateTime` from user input.
fn construction(c: &mut Criterion) {
    let sources = [
        // Offset plus IANA annotation.
        "2025-01-01T00:00:00-05:00[America/New_York]",
        // Annotation only, so the offset is resolved from the provider.
        "2025-07-04T12:30:45[America/New_York]",
        // UTC designator with an annotation.
        "2025-03-09T07:00:00Z[America/New_York]",
        // Sub-second precision.
        "2025-11-02T01:30:00.123456789-05:00[America/New_York]",
    ];

    let mut group = c.benchmark_group("construction");
    group.bench_function("from_utf8", |b| {
        b.iter(|| {
            for source in &sources {
                black_box(parse(black_box(source)));
            }
        })
    });
    group.finish();
}

/// Arithmetic produces a new `ZonedDateTime` without necessarily reading a field,
/// so it is the clearest place an eager cache could regress.
fn arithmetic(c: &mut Criterion) {
    let zdts = zoned_date_times();
    let step = step();
    let day = Duration::new(0, 0, 0, 1, 0, 0, 0, 0, 0, 0).expect("valid duration");

    let mut group = c.benchmark_group("arithmetic");
    group.bench_function("add", |b| {
        b.iter(|| {
            for zdt in &zdts {
                black_box(black_box(zdt).add(&step, None).unwrap());
            }
        })
    });
    // Adding a day is the DST-sensitive case: the result's offset may differ from
    // the receiver's, so the cached offset and ISO date-time must be recomputed.
    group.bench_function("add_one_day", |b| {
        b.iter(|| {
            for zdt in &zdts {
                black_box(black_box(zdt).add(&day, None).unwrap());
            }
        })
    });
    // Parse, add, then read a field — the shape a caller that does real work takes,
    // so the cache is paid for once and used once.
    group.bench_function("parse_add_read", |b| {
        b.iter(|| {
            let zdt = parse(black_box(SOURCE))
                .add(black_box(&step), None)
                .unwrap();
            black_box(zdt.hour())
        })
    });
    group.bench_function("with_timezone", |b| {
        let paris = TimeZone::try_from_str("Europe/Paris").expect("tz must resolve");
        b.iter(|| {
            for zdt in &zdts {
                black_box(black_box(zdt).with_timezone(paris).unwrap());
            }
        })
    });
    group.finish();
}

/// String output reads every field, so it compounds the per-getter cost.
fn to_string(c: &mut Criterion) {
    let zdts = zoned_date_times();

    c.bench_function("to_ixdtf_string", |b| {
        b.iter(|| {
            for zdt in &zdts {
                black_box(
                    black_box(zdt)
                        .to_ixdtf_string(
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        )
                        .unwrap(),
                );
            }
        })
    });
}

criterion_group!(
    benches,
    seven_getters,
    single_getter,
    construction,
    arithmetic,
    to_string
);
criterion_main!(benches);
