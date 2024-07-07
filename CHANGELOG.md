# CHANGELOG

## What's Changed in v0.0.3

* Implement add and subtract methods for Duration by @nekevss in [#74](https://github.com/boa-dev/temporal/pull/74)
* Implement PartialEq and Eq for `Calendar`, `Date`, and `DateTime` by @nekevss in [#75](https://github.com/boa-dev/temporal/pull/75)
* Update duration validation and switch asserts to debug-asserts by @nekevss in [#73](https://github.com/boa-dev/temporal/pull/73)
* Update duration rounding to new algorithms by @nekevss in [#65](https://github.com/boa-dev/temporal/pull/65)
* Remove `CalendarProtocol` and `TimeZoneProtocol` by @jedel1043 in [#66](https://github.com/boa-dev/temporal/pull/66)
* Use groups in dependabot updates by @jedel1043 in [#69](https://github.com/boa-dev/temporal/pull/69)
* Bump num-bigint from 0.4.5 to 0.4.6 by @dependabot[bot] in [#68](https://github.com/boa-dev/temporal/pull/68)
* Bump bitflags from 2.5.0 to 2.6.0 by @dependabot[bot] in [#67](https://github.com/boa-dev/temporal/pull/67)
* Ensure parsing throws with unknown critical annotations by @jedel1043 in [#63](https://github.com/boa-dev/temporal/pull/63)
* Reject `IsoDate` when outside the allowed range by @jedel1043 in [#62](https://github.com/boa-dev/temporal/pull/62)
* Bump rustc-hash from 1.1.0 to 2.0.0 by @dependabot[bot] in [#60](https://github.com/boa-dev/temporal/pull/60)
* Bump icu_calendar from 1.5.1 to 1.5.2 by @dependabot[bot] in [#59](https://github.com/boa-dev/temporal/pull/59)
* Avoid overflowing when calling `NormalizedTimeDuration::add_days` by @jedel1043 in [#61](https://github.com/boa-dev/temporal/pull/61)
* Ensure parsing throws when duplicate calendar is critical by @jedel1043 in [#58](https://github.com/boa-dev/temporal/pull/58)
* Fix rounding when the dividend is smaller than the divisor by @jedel1043 in [#57](https://github.com/boa-dev/temporal/pull/57)
* Implement the `toYearMonth`, `toMonthDay`, and `toDateTime` for `Date` component by @nekevss in [#56](https://github.com/boa-dev/temporal/pull/56)
* Update increment rounding functionality by @nekevss in [#53](https://github.com/boa-dev/temporal/pull/53)
* Patch `(un)balance_relative` to avoid panicking by @jedel1043 in [#48](https://github.com/boa-dev/temporal/pull/48)
* Cleanup rounding increment usages with new struct by @jedel1043 in [#54](https://github.com/boa-dev/temporal/pull/54)
* Add struct to encapsulate invariants of rounding increments by @jedel1043 in [#49](https://github.com/boa-dev/temporal/pull/49)
* Bump icu_calendar from 1.5.0 to 1.5.1 by @dependabot[bot] in [#52](https://github.com/boa-dev/temporal/pull/52)
* Bump tinystr from 0.7.5 to 0.7.6 by @dependabot[bot] in [#51](https://github.com/boa-dev/temporal/pull/51)
* Migrate parsing to `ixdtf` crate by @nekevss in [#50](https://github.com/boa-dev/temporal/pull/50)
* Bump `icu_calendar` to 1.5 by @jedel1043 in [#47](https://github.com/boa-dev/temporal/pull/47)
* Fix method call in days_in_month by @nekevss in [#46](https://github.com/boa-dev/temporal/pull/46)
* Implement add & subtract methods for `DateTime` component by @nekevss in [#45](https://github.com/boa-dev/temporal/pull/45)
* Bump num-bigint from 0.4.4 to 0.4.5 by @dependabot[bot] in [#43](https://github.com/boa-dev/temporal/pull/43)
* Bump num-traits from 0.2.18 to 0.2.19 by @dependabot[bot] in [#42](https://github.com/boa-dev/temporal/pull/42)
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