#ifndef TEMPORAL_PlainDate_D_HPP
#define TEMPORAL_PlainDate_D_HPP

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
namespace capi { struct ParsedDate; }
class ParsedDate;
namespace capi { struct PlainDate; }
class PlainDate;
namespace capi { struct PlainDateTime; }
class PlainDateTime;
namespace capi { struct PlainMonthDay; }
class PlainMonthDay;
namespace capi { struct PlainTime; }
class PlainTime;
namespace capi { struct PlainYearMonth; }
class PlainYearMonth;
namespace capi { struct Provider; }
class Provider;
namespace capi { struct ZonedDateTime; }
class ZonedDateTime;
struct DifferenceSettings;
struct I128Nanoseconds;
struct PartialDate;
struct TemporalError;
struct TimeZone;
class AnyCalendarKind;
class ArithmeticOverflow;
class DisplayCalendar;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PlainDate;
} // namespace capi
} // namespace

namespace temporal {
class PlainDate {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> try_new_constrain(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> try_new(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> try_new_with_overflow(int32_t year, uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_parsed(const temporal::ParsedDate& parsed);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline std::unique_ptr<temporal::PlainDate> with_calendar(temporal::AnyCalendarKind calendar) const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline const temporal::Calendar& calendar() const DIPLOMAT_LIFETIME_BOUND;

  inline bool is_valid() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> add(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> subtract(const temporal::Duration& duration, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::PlainDate& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::PlainDate& other, temporal::DifferenceSettings settings) const;

  inline bool equals(const temporal::PlainDate& other) const;

  inline static int8_t compare(const temporal::PlainDate& one, const temporal::PlainDate& two);

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

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDateTime>, temporal::TemporalError> to_plain_date_time(const temporal::PlainTime* time) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> to_plain_month_day() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> to_plain_year_month() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time(temporal::TimeZone time_zone, const temporal::PlainTime* time) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time_with_provider(temporal::TimeZone time_zone, const temporal::PlainTime* time, const temporal::Provider& p) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_utc() const;

  inline std::string to_ixdtf_string(temporal::DisplayCalendar display_calendar) const;
  template<typename W>
  inline void to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable_output) const;

  inline std::unique_ptr<temporal::PlainDate> clone() const;

    inline const temporal::capi::PlainDate* AsFFI() const;
    inline temporal::capi::PlainDate* AsFFI();
    inline static const temporal::PlainDate* FromFFI(const temporal::capi::PlainDate* ptr);
    inline static temporal::PlainDate* FromFFI(temporal::capi::PlainDate* ptr);
    inline static void operator delete(void* ptr);
private:
    PlainDate() = delete;
    PlainDate(const temporal::PlainDate&) = delete;
    PlainDate(temporal::PlainDate&&) noexcept = delete;
    PlainDate operator=(const temporal::PlainDate&) = delete;
    PlainDate operator=(temporal::PlainDate&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_PlainDate_D_HPP
