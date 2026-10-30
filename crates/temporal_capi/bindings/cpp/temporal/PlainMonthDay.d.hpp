#ifndef TEMPORAL_PlainMonthDay_D_HPP
#define TEMPORAL_PlainMonthDay_D_HPP

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
namespace capi { struct ParsedDate; }
class ParsedDate;
namespace capi { struct PlainDate; }
class PlainDate;
namespace capi { struct PlainMonthDay; }
class PlainMonthDay;
namespace capi { struct Provider; }
class Provider;
struct PartialDate;
struct TemporalError;
struct TimeZone;
class AnyCalendarKind;
class ArithmeticOverflow;
class DisplayCalendar;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PlainMonthDay;
} // namespace capi
} // namespace

namespace temporal {
class PlainMonthDay {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> try_new_with_overflow(uint8_t month, uint8_t day, temporal::AnyCalendarKind calendar, temporal::ArithmeticOverflow overflow, std::optional<int32_t> ref_year);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> from_partial(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> from_parsed(const temporal::ParsedDate& parsed);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> with(temporal::PartialDate partial, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline bool equals(const temporal::PlainMonthDay& other) const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainMonthDay>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline uint8_t day() const;

  inline const temporal::Calendar& calendar() const DIPLOMAT_LIFETIME_BOUND;

  inline uint8_t month() const;

  inline std::string month_code() const;
  template<typename W>
  inline void month_code_write(W& writeable_output) const;

  inline int32_t reference_year() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainDate>, temporal::TemporalError> to_plain_date(std::optional<temporal::PartialDate> year) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_utc() const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for(temporal::TimeZone time_zone) const;

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_with_provider(temporal::TimeZone time_zone, const temporal::Provider& p) const;

  inline std::string to_ixdtf_string(temporal::DisplayCalendar display_calendar) const;
  template<typename W>
  inline void to_ixdtf_string_write(temporal::DisplayCalendar display_calendar, W& writeable_output) const;

  inline std::unique_ptr<temporal::PlainMonthDay> clone() const;

    inline const temporal::capi::PlainMonthDay* AsFFI() const;
    inline temporal::capi::PlainMonthDay* AsFFI();
    inline static const temporal::PlainMonthDay* FromFFI(const temporal::capi::PlainMonthDay* ptr);
    inline static temporal::PlainMonthDay* FromFFI(temporal::capi::PlainMonthDay* ptr);
    inline static void operator delete(void* ptr);
private:
    PlainMonthDay() = delete;
    PlainMonthDay(const temporal::PlainMonthDay&) = delete;
    PlainMonthDay(temporal::PlainMonthDay&&) noexcept = delete;
    PlainMonthDay operator=(const temporal::PlainMonthDay&) = delete;
    PlainMonthDay operator=(temporal::PlainMonthDay&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_PlainMonthDay_D_HPP
