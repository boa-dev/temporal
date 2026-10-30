#ifndef TEMPORAL_Calendar_D_HPP
#define TEMPORAL_Calendar_D_HPP

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
struct TemporalError;
class AnyCalendarKind;
} // namespace temporal



namespace temporal {
namespace capi {
    struct Calendar;
} // namespace capi
} // namespace

namespace temporal {
/**
 * By and large one should not need to construct this type; it mostly exists
 * to represent calendar data found on other Temporal types, obtained via `.calendar()`.
 */
class Calendar {
public:

  inline static std::unique_ptr<temporal::Calendar> try_new_constrain(temporal::AnyCalendarKind kind);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Calendar>, temporal::TemporalError> from_utf8(std::string_view s);

  inline bool is_iso() const;

  inline std::string_view identifier() const;

  /**
   * Returns the kind of this calendar
   */
  inline temporal::AnyCalendarKind kind() const;

    inline const temporal::capi::Calendar* AsFFI() const;
    inline temporal::capi::Calendar* AsFFI();
    inline static const temporal::Calendar* FromFFI(const temporal::capi::Calendar* ptr);
    inline static temporal::Calendar* FromFFI(temporal::capi::Calendar* ptr);
    inline static void operator delete(void* ptr);
private:
    Calendar() = delete;
    Calendar(const temporal::Calendar&) = delete;
    Calendar(temporal::Calendar&&) noexcept = delete;
    Calendar operator=(const temporal::Calendar&) = delete;
    Calendar operator=(temporal::Calendar&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_Calendar_D_HPP
