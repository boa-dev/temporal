#ifndef TEMPORAL_ZonedDateTime_D_HPP
#define TEMPORAL_ZonedDateTime_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace temporal {
namespace capi { struct Calendar; }
class Calendar;
namespace capi { struct Duration; }
class Duration;
namespace capi { struct Instant; }
class Instant;
namespace capi { struct ParsedZonedDateTime; }
class ParsedZonedDateTime;
namespace capi { struct PlainDate; }
class PlainDate;
namespace capi { struct PlainDateTime; }
class PlainDateTime;
namespace capi { struct PlainTime; }
class PlainTime;
namespace capi { struct Provider; }
class Provider;
namespace capi { struct ZonedDateTime; }
class ZonedDateTime;
struct DifferenceSettings;
struct I128Nanoseconds;
struct PartialZonedDateTime;
struct RoundingOptions;
struct TemporalError;
struct TimeZone;
struct ToStringRoundingOptions;
class AnyCalendarKind;
class ArithmeticOverflow;
class Disambiguation;
class DisplayCalendar;
class DisplayOffset;
class DisplayTimeZone;
class OffsetDisambiguation;
class TransitionDirection;
} // namespace temporal



namespace temporal {
namespace capi {
    struct ZonedDateTime;
} // namespace capi
} // namespace

namespace temporal {
class ZonedDateTime {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> try_new(temporal::I128Nanoseconds nanosecond, temporal::AnyCalendarKind calendar, temporal::TimeZone time_zone);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> try_new_with_provider(temporal::I128Nanoseconds nanosecond, temporal::AnyCalendarKind calendar, temporal::TimeZone time_zone, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_partial(temporal::PartialZonedDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_partial_with_provider(temporal::PartialZonedDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_parsed(const temporal::ParsedZonedDateTime& parsed, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_option);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_parsed_with_provider(const temporal::ParsedZonedDateTime& parsed, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_option, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_utf8(std::string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_utf8_with_provider(std::string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_utf16(std::u16string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_utf16_with_provider(std::u16string_view s, temporal::Disambiguation disambiguation, temporal::OffsetDisambiguation offset_disambiguation, const temporal::Provider& p);

  inline int64_t epoch_milliseconds() const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p);

  inline temporal::I128Nanoseconds epoch_nanoseconds() const;

  inline int64_t offset_nanoseconds() const;

  inline std::unique_ptr<temporal::Instant> to_instant() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with(temporal::PartialZonedDateTime partial, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with_with_provider(temporal::PartialZonedDateTime partial, std::optional<temporal::Disambiguation> disambiguation, std::optional<temporal::OffsetDisambiguation> offset_option, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with_timezone(temporal::TimeZone zone) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with_timezone_with_provider(temporal::TimeZone zone, const temporal::Provider& p) const;

  inline temporal::TimeZone timezone() const;

  inline int8_t compare_instant(const temporal::ZonedDateTime& other) const;

  inline bool equals(const temporal::ZonedDateTime& other) const;

  inline temporal::diplomat::result<bool, temporal::TemporalError> equals_with_provider(const temporal::ZonedDateTime& other, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> offset() const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> offset_write(W& writeable_output) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> start_of_day() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> start_of_day_with_provider(const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> get_time_zone_transition(temporal::TransitionDirection direction) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> get_time_zone_transition_with_provider(temporal::TransitionDirection direction, const temporal::Provider& p) const;

  inline temporal::diplomat::result<double, temporal::TemporalError> hours_in_day() const;

  inline temporal::diplomat::result<double, temporal::TemporalError> hours_in_day_with_provider(const temporal::Provider& p) const;

  inline std::unique_ptr<temporal::PlainDateTime> to_plain_datetime() const;

  inline std::unique_ptr<temporal::PlainDate> to_plain_date() const;

  inline std::unique_ptr<temporal::PlainTime> to_plain_time() const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_write(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, W& writeable_output) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string_with_provider(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, const temporal::Provider& p) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_with_provider_write(temporal::DisplayOffset display_offset, temporal::DisplayTimeZone display_timezone, temporal::DisplayCalendar display_calendar, temporal::ToStringRoundingOptions options, const temporal::Provider& p, W& writeable_output) const;

  inline std::unique_ptr<temporal::ZonedDateTime> with_calendar(temporal::AnyCalendarKind calendar) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with_plain_time(const temporal::PlainTime* time) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> with_plain_time_and_provider(const temporal::PlainTime* time, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> add_with_provider(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> subtract_with_provider(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until_with_provider(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since_with_provider(const temporal::ZonedDateTime& other, temporal::DifferenceSettings settings, const temporal::Provider& p) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> round(temporal::RoundingOptions options) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> round_with_provider(temporal::RoundingOptions options, const temporal::Provider& p) const;

  inline uint8_t hour() const;

  inline uint8_t minute() const;

  inline uint8_t second() const;

  inline uint16_t millisecond() const;

  inline uint16_t microsecond() const;

  inline uint16_t nanosecond() const;

  inline const temporal::Calendar& calendar() const DIPLOMAT_LIFETIME_BOUND;

  inline int32_t year() const;

  inline uint8_t month() const;

  inline std::string month_code() const;
  template<typename W>
  inline void month_code_write(W& writeable_output) const;

  inline uint8_t day() const;

  inline uint16_t day_of_week() const;

  inline uint16_t day_of_year() const;

  inline std::optional<uint8_t> week_of_year() const;

  inline std::optional<int32_t> year_of_week() const;

  inline uint16_t days_in_week() const;

  inline uint16_t days_in_month() const;

  inline uint16_t days_in_year() const;

  inline uint16_t months_in_year() const;

  inline bool in_leap_year() const;

  inline std::string era() const;
  template<typename W>
  inline void era_write(W& writeable_output) const;

  inline std::optional<int32_t> era_year() const;

  inline std::unique_ptr<temporal::ZonedDateTime> clone() const;

    inline const temporal::capi::ZonedDateTime* AsFFI() const;
    inline temporal::capi::ZonedDateTime* AsFFI();
    inline static const temporal::ZonedDateTime* FromFFI(const temporal::capi::ZonedDateTime* ptr);
    inline static temporal::ZonedDateTime* FromFFI(temporal::capi::ZonedDateTime* ptr);
    inline static void operator delete(void* ptr);
private:
    ZonedDateTime() = delete;
    ZonedDateTime(const temporal::ZonedDateTime&) = delete;
    ZonedDateTime(temporal::ZonedDateTime&&) noexcept = delete;
    ZonedDateTime operator=(const temporal::ZonedDateTime&) = delete;
    ZonedDateTime operator=(temporal::ZonedDateTime&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_ZonedDateTime_D_HPP
