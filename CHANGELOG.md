## What's Changed in v0.0.4

* bump release by @jasonwilliams in [#120](https://github.com/boa-dev/temporal/pull/120)
* Add an `EpochNanosecond` new type by @nekevss in [#116](https://github.com/boa-dev/temporal/pull/116)
* Migrate to `web_time::SystemTime` for `wasm32-unknown-unknown` targets by @nekevss in [#118](https://github.com/boa-dev/temporal/pull/118)
* Bug fixes and more implementation by @jasonwilliams in [#110](https://github.com/boa-dev/temporal/pull/110)
* Some `Error` optimizations by @CrazyboyQCD in [#112](https://github.com/boa-dev/temporal/pull/112)
* Add `from_partial` methods to `PlainTime`, `PlainDate`, and `PlainDateTime` by @nekevss in [#106](https://github.com/boa-dev/temporal/pull/106)
* Implement `ZonedDateTime`'s add and subtract methods by @nekevss in [#102](https://github.com/boa-dev/temporal/pull/102)
* Add matrix links to README and some layout adjustments by @nekevss in [#108](https://github.com/boa-dev/temporal/pull/108)
* Stub out `tzdb` support for Windows and POSIX tz string by @nekevss in [#100](https://github.com/boa-dev/temporal/pull/100)
* Stub out tzdb support to unblock `Now` and `ZonedDateTime` by @nekevss in [#99](https://github.com/boa-dev/temporal/pull/99)
* Remove num-bigint dependency and rely on primitives by @nekevss in [#103](https://github.com/boa-dev/temporal/pull/103)
* Move to no_std by @Manishearth in [#101](https://github.com/boa-dev/temporal/pull/101)
* General API cleanup and adjustments by @nekevss in [#97](https://github.com/boa-dev/temporal/pull/97)
* Update README.md by @jasonwilliams in [#96](https://github.com/boa-dev/temporal/pull/96)
* Refactor `TemporalFields` into `CalendarFields` by @nekevss in [#95](https://github.com/boa-dev/temporal/pull/95)
* Patch for partial records by @nekevss in [#94](https://github.com/boa-dev/temporal/pull/94)
* Add `PartialTime` and `PartialDateTime` with corresponding `with` methods. by @nekevss in [#92](https://github.com/boa-dev/temporal/pull/92)
* Implement `MonthCode`, `PartialDate`, and `Date::with` by @nekevss in [#89](https://github.com/boa-dev/temporal/pull/89)
* Add is empty for partialDuration by @jasonwilliams in [#90](https://github.com/boa-dev/temporal/pull/90)
* Fix lints for rustc 1.80.0 by @jedel1043 in [#91](https://github.com/boa-dev/temporal/pull/91)
* adding methods for yearMonth and MonthDay by @jasonwilliams in [#44](https://github.com/boa-dev/temporal/pull/44)
* Implement `DateTime` round method by @nekevss in [#88](https://github.com/boa-dev/temporal/pull/88)
* Update `Duration` types to use a `FiniteF64` instead of `f64` primitive. by @nekevss in [#86](https://github.com/boa-dev/temporal/pull/86)
* Refactor `TemporalFields` interface and add `FieldsKey` enum by @nekevss in [#87](https://github.com/boa-dev/temporal/pull/87)
* Updates to instant and its methods by @nekevss in [#85](https://github.com/boa-dev/temporal/pull/85)
* Implement compare functionality and some more traits by @nekevss in [#82](https://github.com/boa-dev/temporal/pull/82)
* Implement `DateTime` diffing methods `Until` and `Since` by @nekevss in [#83](https://github.com/boa-dev/temporal/pull/83)
* Add `with_*` methods to `Date` and `DateTime` by @nekevss in [#84](https://github.com/boa-dev/temporal/pull/84)
* Add some missing trait implementations by @nekevss in [#81](https://github.com/boa-dev/temporal/pull/81)
* chore(dependabot): bump zerovec-derive from 0.10.2 to 0.10.3 by @dependabot[bot] in [#80](https://github.com/boa-dev/temporal/pull/80)
* Add prefix option to commit-message by @nekevss in [#79](https://github.com/boa-dev/temporal/pull/79)
* Add commit-message prefix to dependabot by @nekevss in [#77](https://github.com/boa-dev/temporal/pull/77)
* Bump zerovec from 0.10.2 to 0.10.4 by @dependabot[bot] in [#78](https://github.com/boa-dev/temporal/pull/78)

## New Contributors
* @jasonwilliams made their first contribution in [#120](https://github.com/boa-dev/temporal/pull/120)
* @CrazyboyQCD made their first contribution in [#112](https://github.com/boa-dev/temporal/pull/112)
* @Manishearth made their first contribution in [#101](https://github.com/boa-dev/temporal/pull/101)

**Full Changelog**: https://github.com/boa-dev/temporal/compare/v0.0.3...v0.0.4

# CHANGELOG

## What's Changed in v0.0.3

* Implement add and subtract methods for Duration by @nekevss in [#74](https://github.com/boa-dev/temporal/pull/74)
* Implement PartialEq and Eq for `Calendar`, `Date`, and `DateTime` by @nekevss in [#75](https://github.com/boa-dev/temporal/pull/75)
* Update duration validation and switch asserts to debug-asserts by @nekevss in [#73](https://github.com/boa-dev/temporal/pull/73)
* Update duration rounding to new algorithms by @nekevss in [#65](https://github.com/boa-dev/temporal/pull/65)
* Remove `CalendarProtocol` and `TimeZoneProtocol` by @jedel1043 in [#66](https://github.com/boa-dev/temporal/pull/66)
* Use groups in dependabot updates by @jedel1043 in [#69](https://github.com/boa-dev/temporal/pull/69)
* Ensure parsing throws with unknown critical annotations by @jedel1043 in [#63](https://github.com/boa-dev/temporal/pull/63)
* Reject `IsoDate` when outside the allowed range by @jedel1043 in [#62](https://github.com/boa-dev/temporal/pull/62)
* Avoid overflowing when calling `NormalizedTimeDuration::add_days` by @jedel1043 in [#61](https://github.com/boa-dev/temporal/pull/61)
* Ensure parsing throws when duplicate calendar is critical by @jedel1043 in [#58](https://github.com/boa-dev/temporal/pull/58)
* Fix rounding when the dividend is smaller than the divisor by @jedel1043 in [#57](https://github.com/boa-dev/temporal/pull/57)
* Implement the `toYearMonth`, `toMonthDay`, and `toDateTime` for `Date` component by @nekevss in [#56](https://github.com/boa-dev/temporal/pull/56)
* Update increment rounding functionality by @nekevss in [#53](https://github.com/boa-dev/temporal/pull/53)
* Patch `(un)balance_relative` to avoid panicking by @jedel1043 in [#48](https://github.com/boa-dev/temporal/pull/48)
* Cleanup rounding increment usages with new struct by @jedel1043 in [#54](https://github.com/boa-dev/temporal/pull/54)
* Add struct to encapsulate invariants of rounding increments by @jedel1043 in [#49](https://github.com/boa-dev/temporal/pull/49)
* Migrate parsing to `ixdtf` crate by @nekevss in [#50](https://github.com/boa-dev/temporal/pull/50)
* Fix method call in days_in_month by @nekevss in [#46](https://github.com/boa-dev/temporal/pull/46)
* Implement add & subtract methods for `DateTime` component by @nekevss in [#45](https://github.com/boa-dev/temporal/pull/45)
* Fix panics when no relative_to is supplied to round by @nekevss in [#40](https://github.com/boa-dev/temporal/pull/40)
* Implement Time's until and since methods by @nekevss in [#36](https://github.com/boa-dev/temporal/pull/36)
* Implements `Date`'s `add`, `subtract`, `until`, and `since` methods by @nekevss in [#35](https://github.com/boa-dev/temporal/pull/35)
* Fix clippy lints and bump bitflags version by @nekevss in [#38](https://github.com/boa-dev/temporal/pull/38)

**Full Changelog**: https://github.com/boa-dev/temporal/compare/v0.0.2...v0.0.3

## What's Changed in v0.0.2

# [0.0.2 (2024-03-04)](https://github.com/boa-dev/temporal/compare/v0.0.1...v0.0.2)

### Enhancements

* Fix loop in `diff_iso_date` by @nekevss in https://github.com/boa-dev/temporal/pull/31
* Remove unnecessary iterations by @nekevss in https://github.com/boa-dev/temporal/pull/30

**Full Changelog**: https://github.com/boa-dev/temporal/compare/v0.0.1...v0.0.2

# [0.0.1 (2024-02-25)](https://github.com/boa-dev/temporal/commits/v0.0.1)

### Enhancements
* Add blank and negated + small adjustments by @nekevss in https://github.com/boa-dev/temporal/pull/17
* Simplify Temporal APIs by @jedel1043 in https://github.com/boa-dev/temporal/pull/18
* Implement `Duration` normalization - Part 1 by @nekevss in https://github.com/boa-dev/temporal/pull/20
* Duration Normalization - Part 2 by @nekevss in https://github.com/boa-dev/temporal/pull/23
* Add `non_exhaustive` attribute to component structs by @nekevss in https://github.com/boa-dev/temporal/pull/25
* Implement `Duration::round` and some general updates/fixes by @nekevss in https://github.com/boa-dev/temporal/pull/24

### Documentation
* Adding a `docs` directory by @nekevss in https://github.com/boa-dev/temporal/pull/16
* Build out README and CONTRIBUTING docs by @nekevss in https://github.com/boa-dev/temporal/pull/21

### Other Changes
* Port `boa_temporal` to new `temporal` crate by @nekevss in https://github.com/boa-dev/temporal/pull/1
* Add CI and rename license by @jedel1043 in https://github.com/boa-dev/temporal/pull/3
* Create LICENSE-Apache by @jedel1043 in https://github.com/boa-dev/temporal/pull/6
* Setup publish CI by @jedel1043 in https://github.com/boa-dev/temporal/pull/26
* Remove keywords from Cargo.toml by @jedel1043 in https://github.com/boa-dev/temporal/pull/28

## New Contributors
* @nekevss made their first contribution in https://github.com/boa-dev/temporal/pull/1
* @jedel1043 made their first contribution in https://github.com/boa-dev/temporal/pull/3

**Full Changelog**: https://github.com/boa-dev/temporal/commits/v0.0.1