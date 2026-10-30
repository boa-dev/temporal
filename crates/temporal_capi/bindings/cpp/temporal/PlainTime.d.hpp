#ifndef TEMPORAL_PlainTime_D_HPP
#define TEMPORAL_PlainTime_D_HPP

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
namespace capi { struct Duration; }
class Duration;
namespace capi { struct PlainTime; }
class PlainTime;
namespace capi { struct Provider; }
class Provider;
struct DifferenceSettings;
struct I128Nanoseconds;
struct PartialTime;
struct RoundingOptions;
struct TemporalError;
struct TimeZone;
struct ToStringRoundingOptions;
class ArithmeticOverflow;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PlainTime;
} // namespace capi
} // namespace

namespace temporal {
class PlainTime {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> try_new_constrain(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> try_new(uint8_t hour, uint8_t minute, uint8_t second, uint16_t millisecond, uint16_t microsecond, uint16_t nanosecond);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_partial(temporal::PartialTime partial, std::optional<temporal::ArithmeticOverflow> overflow);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_epoch_milliseconds(int64_t ms, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_epoch_milliseconds_with_provider(int64_t ms, temporal::TimeZone tz, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_epoch_nanoseconds(temporal::I128Nanoseconds ns, temporal::TimeZone tz);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_epoch_nanoseconds_with_provider(temporal::I128Nanoseconds ns, temporal::TimeZone tz, const temporal::Provider& p);

  inline temporal::diplomat::result<int64_t, temporal::TemporalError> epoch_ms_for_utc() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> with(temporal::PartialTime partial, std::optional<temporal::ArithmeticOverflow> overflow) const;

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline uint8_t hour() const;

  inline uint8_t minute() const;

  inline uint8_t second() const;

  inline uint16_t millisecond() const;

  inline uint16_t microsecond() const;

  inline uint16_t nanosecond() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> add(const temporal::Duration& duration) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> subtract(const temporal::Duration& duration) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::PlainTime& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::PlainTime& other, temporal::DifferenceSettings settings) const;

  inline bool equals(const temporal::PlainTime& other) const;

  inline static int8_t compare(const temporal::PlainTime& one, const temporal::PlainTime& two);

  inline temporal::diplomat::result<std::unique_ptr<temporal::PlainTime>, temporal::TemporalError> round(temporal::RoundingOptions options) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string(temporal::ToStringRoundingOptions options) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_write(temporal::ToStringRoundingOptions options, W& writeable_output) const;

  inline std::unique_ptr<temporal::PlainTime> clone() const;

    inline const temporal::capi::PlainTime* AsFFI() const;
    inline temporal::capi::PlainTime* AsFFI();
    inline static const temporal::PlainTime* FromFFI(const temporal::capi::PlainTime* ptr);
    inline static temporal::PlainTime* FromFFI(temporal::capi::PlainTime* ptr);
    inline static void operator delete(void* ptr);
private:
    PlainTime() = delete;
    PlainTime(const temporal::PlainTime&) = delete;
    PlainTime(temporal::PlainTime&&) noexcept = delete;
    PlainTime operator=(const temporal::PlainTime&) = delete;
    PlainTime operator=(temporal::PlainTime&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_PlainTime_D_HPP
