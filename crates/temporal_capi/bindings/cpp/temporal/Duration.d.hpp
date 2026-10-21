#ifndef TEMPORAL_Duration_D_HPP
#define TEMPORAL_Duration_D_HPP

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
namespace capi { struct Provider; }
class Provider;
struct PartialDuration;
struct RelativeTo;
struct RoundingOptions;
struct TemporalError;
struct ToStringRoundingOptions;
class Sign;
class Unit;
} // namespace temporal



namespace temporal {
namespace capi {
    struct Duration;
} // namespace capi
} // namespace

namespace temporal {
class Duration {
public:

  /**
   * Temporary API until v8 can move off of it
   */
  inline static temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> create(int64_t years, int64_t months, int64_t weeks, int64_t days, int64_t hours, int64_t minutes, int64_t seconds, int64_t milliseconds, double microseconds, double nanoseconds);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> try_new(int64_t years, int64_t months, int64_t weeks, int64_t days, int64_t hours, int64_t minutes, int64_t seconds, int64_t milliseconds, double microseconds, double nanoseconds);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> from_partial_duration(temporal::PartialDuration partial);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline bool is_time_within_range() const;

  inline int64_t years() const;

  inline int64_t months() const;

  inline int64_t weeks() const;

  inline int64_t days() const;

  inline int64_t hours() const;

  inline int64_t minutes() const;

  inline int64_t seconds() const;

  inline int64_t milliseconds() const;

  inline double microseconds() const;

  inline double nanoseconds() const;

  inline temporal::Sign sign() const;

  inline bool is_zero() const;

  inline std::unique_ptr<temporal::Duration> abs() const;

  inline std::unique_ptr<temporal::Duration> negated() const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> add(const temporal::Duration& other) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> subtract(const temporal::Duration& other) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_string(temporal::ToStringRoundingOptions options) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_string_write(temporal::ToStringRoundingOptions options, W& writeable_output) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> round(temporal::RoundingOptions options, temporal::RelativeTo relative_to) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> round_with_provider(temporal::RoundingOptions options, temporal::RelativeTo relative_to, const temporal::Provider& p) const;

  inline temporal::diplomat::result<int8_t, temporal::TemporalError> compare(const temporal::Duration& other, temporal::RelativeTo relative_to) const;

  inline temporal::diplomat::result<int8_t, temporal::TemporalError> compare_with_provider(const temporal::Duration& other, temporal::RelativeTo relative_to, const temporal::Provider& p) const;

  inline temporal::diplomat::result<double, temporal::TemporalError> total(temporal::Unit unit, temporal::RelativeTo relative_to) const;

  inline temporal::diplomat::result<double, temporal::TemporalError> total_with_provider(temporal::Unit unit, temporal::RelativeTo relative_to, const temporal::Provider& p) const;

  inline std::unique_ptr<temporal::Duration> clone() const;

    inline const temporal::capi::Duration* AsFFI() const;
    inline temporal::capi::Duration* AsFFI();
    inline static const temporal::Duration* FromFFI(const temporal::capi::Duration* ptr);
    inline static temporal::Duration* FromFFI(temporal::capi::Duration* ptr);
    inline static void operator delete(void* ptr);
private:
    Duration() = delete;
    Duration(const temporal::Duration&) = delete;
    Duration(temporal::Duration&&) noexcept = delete;
    Duration operator=(const temporal::Duration&) = delete;
    Duration operator=(temporal::Duration&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_Duration_D_HPP
