# calendar_equations

Gregorian and ISO-calendar date equations shared across the `temporal`
workspace (`temporal`, `timezone_provider`, `zoneinfo_rs`).

This crate has no dependencies and performs no I/O; it converts between
epoch days/milliseconds and Gregorian year/month/day, and computes leap
years, weekdays, and ISO month lengths. The Gregorian conversion is based
on Cassio Neri and Lorenz Schneider's paper, [Euclidean affine functions
and their application to calendar
algorithms](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172).

## Why a separate crate?

This logic was previously duplicated between `timezone_provider` (as a
`#[doc(hidden)]` module) and `zoneinfo_rs` (with slightly different
signatures). Consolidating it here removes the duplication and lets pure
calendar math be consumed without pulling in the heavier tzdb/TZif-parsing
dependencies of `timezone_provider`.
