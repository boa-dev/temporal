#ifndef TEMPORAL_DateDuration_D_HPP
#define TEMPORAL_DateDuration_D_HPP

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
namespace capi { struct DateDuration; }
class DateDuration;
struct TemporalError;
class Sign;
} // namespace temporal



namespace temporal {
namespace capi {
    struct DateDuration;
} // namespace capi
} // namespace

namespace temporal {
class DateDuration {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::DateDuration>, temporal::TemporalError> try_new(int64_t years, int64_t months, int64_t weeks, int64_t days);

  inline std::unique_ptr<temporal::DateDuration> abs() const;

  inline std::unique_ptr<temporal::DateDuration> negated() const;

  inline temporal::Sign sign() const;

    inline const temporal::capi::DateDuration* AsFFI() const;
    inline temporal::capi::DateDuration* AsFFI();
    inline static const temporal::DateDuration* FromFFI(const temporal::capi::DateDuration* ptr);
    inline static temporal::DateDuration* FromFFI(temporal::capi::DateDuration* ptr);
    inline static void operator delete(void* ptr);
private:
    DateDuration() = delete;
    DateDuration(const temporal::DateDuration&) = delete;
    DateDuration(temporal::DateDuration&&) noexcept = delete;
    DateDuration operator=(const temporal::DateDuration&) = delete;
    DateDuration operator=(temporal::DateDuration&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_DateDuration_D_HPP
