#ifndef TEMPORAL_PlainYearMonth_D_HPP
#define TEMPORAL_PlainYearMonth_D_HPP

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
namespace capi { struct PlainYearMonth; }
class PlainYearMonth;
namespace capi { struct Provider; }
class Provider;
struct DifferenceSettings;
struct PartialDate;
struct TemporalError;
struct TimeZone;
class AnyCalendarKind;
class ArithmeticOverflow;
class DisplayCalendar;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PlainYearMonth;
} // namespace capi
} // namespace

namespace temporal {
class PlainYearMonth {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> try_new_with_overflow(int32_t year, uint8_t month, std::optional<uint8_t> reference_day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> from_parsed(const temporal::ParsedDate& parsed);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline int32_t year() const;

  inline uint8_t month() const;

  inline std::string month_code() const;
  template<typename W>
  inline void month_code_write(W& writeable_output) const;

  inline bool in_leap_year() const;

  inline uint16_t days_in_month() const;

  inline uint16_t days_in_year() const;

  inline uint16_t months_in_year() const;

  inline std::string era() const;
  template<typename W>
  inline void era_write(W& writeable_output) const;

  inline std::optional<int32_t> era_year() const;

  inline uint8_t reference_day() const;

  inline const temporal::Calendar& calendar() const DIPLOMAT_LIFETIME_BOUND;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> add(const temporal::Duration& duration, temporal::ArithmeticOverflow overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainYearMonth>, temporal::TemporalError> subtract(const temporal::Duration& duration, temporal::ArithmeticOverflow overflow) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::PlainYearMonth& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::PlainYearMonth& other, temporal::DifferenceSettings settings) const;

  inline bool equals(const temporal::PlainYearMonth& other) const;

  inline static int8_t compare(const temporal::PlainYearMonth& one, const temporal::PlainYearMonth& two);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> to_plain_date(std::optional<temporal::PartialDate> day) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_utc() const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for(temporal::TimeZone time_zone) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_with_provider(temporal::TimeZone time_zone, const temporal::Provider& p) const;

  inline std::string to_ixdtf_string(temporal::DisplayCalendar display_calendar) const;
  template<typename W>
  inline void to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable_output) const;

  inline std::unique_ptr<temporal::PlainYearMonth> clone() const;

    inline const temporal::capi::PlainYearMonth* AsFFI() const;
    inline temporal::capi::PlainYearMonth* AsFFI();
    inline static const temporal::PlainYearMonth* FromFFI(const temporal::capi::PlainYearMonth* ptr);
    inline static temporal::PlainYearMonth* FromFFI(temporal::capi::PlainYearMonth* ptr);
    inline static void operator delete(void* ptr);
private:
    PlainYearMonth() = delete;
    PlainYearMonth(const temporal::PlainYearMonth&) = delete;
    PlainYearMonth(temporal::PlainYearMonth&&) noexcept = delete;
    PlainYearMonth operator=(const temporal::PlainYearMonth&) = delete;
    PlainYearMonth operator=(temporal::PlainYearMonth&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_PlainYearMonth_D_HPP
