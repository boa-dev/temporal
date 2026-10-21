#ifndef TEMPORAL_PlainDateTime_D_HPP
#define TEMPORAL_PlainDateTime_D_HPP

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
namespace capi { struct ParsedDateTime; }
class ParsedDateTime;
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
struct PartialDateTime;
struct RoundingOptions;
struct TemporalError;
struct TimeZone;
struct ToStringRoundingOptions;
class AnyCalendarKind;
class ArithmeticOverflow;
class Disambiguation;
class DisplayCalendar;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PlainDateTime;
} // namespace capi
} // namespace

namespace temporal {
class PlainDateTime {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> try_new_constrain(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::AnyCalendarKind calendar);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> try_new(int32_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond, temporal::AnyCalendarKind calendar);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_partial(temporal::PartialDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_parsed(const temporal::ParsedDateTime& parsed);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> with(temporal::PartialDateTime partial, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> with_time(const temporal::PlainTime* time) const;

  inline std::unique_ptr<temporal::PlainDateTime> with_calendar(temporal::AnyCalendarKind calendar) const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> from_utf16(std::u16string_view s);

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

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::PlainDateTime& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::PlainDateTime& other, temporal::DifferenceSettings settings) const;

  inline bool equals(const temporal::PlainDateTime& other) const;

  inline static int8_t compare(const temporal::PlainDateTime& one, const temporal::PlainDateTime& two);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> round(temporal::RoundingOptions options) const;

  inline std::unique_ptr<temporal::PlainDate> to_plain_date() const;

  inline std::unique_ptr<temporal::PlainTime> to_plain_time() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time(temporal::TimeZone time_zone, temporal::Disambiguation disambiguation) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time_with_provider(temporal::TimeZone time_zone, temporal::Disambiguation disambiguation, const temporal::Provider& p) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_utc() const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string(temporal::ToStringRoundingOptions options, temporal::DisplayCalendar display_calendar) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_write(temporal::ToStringRoundingOptions options, temporal::DisplayCalendar display_calendar, W& writeable_output) const;

  inline std::unique_ptr<temporal::PlainDateTime> clone() const;

    inline const temporal::capi::PlainDateTime* AsFFI() const;
    inline temporal::capi::PlainDateTime* AsFFI();
    inline static const temporal::PlainDateTime* FromFFI(const temporal::capi::PlainDateTime* ptr);
    inline static temporal::PlainDateTime* FromFFI(temporal::capi::PlainDateTime* ptr);
    inline static void operator delete(void* ptr);
private:
    PlainDateTime() = delete;
    PlainDateTime(const temporal::PlainDateTime&) = delete;
    PlainDateTime(temporal::PlainDateTime&&) noexcept = delete;
    PlainDateTime operator=(const temporal::PlainDateTime&) = delete;
    PlainDateTime operator=(temporal::PlainDateTime&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_PlainDateTime_D_HPP
