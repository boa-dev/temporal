#ifndef TEMPORAL_AnyCalendarKind_D_HPP
#define TEMPORAL_AnyCalendarKind_D_HPP

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
class AnyCalendarKind;
} // namespace temporal



namespace temporal {
namespace capi {
    enum AnyCalendarKind {
      AnyCalendarKind_Iso = 0,
      AnyCalendarKind_Buddhist = 1,
      AnyCalendarKind_Chinese = 2,
      AnyCalendarKind_Coptic = 3,
      AnyCalendarKind_Dangi = 4,
      AnyCalendarKind_Ethiopian = 5,
      AnyCalendarKind_EthiopianAmeteAlem = 6,
      AnyCalendarKind_Gregorian = 7,
      AnyCalendarKind_Hebrew = 8,
      AnyCalendarKind_Indian = 9,
      AnyCalendarKind_HijriTabularTypeIIFriday = 10,
      AnyCalendarKind_HijriTabularTypeIIThursday = 11,
      AnyCalendarKind_HijriUmmAlQura = 12,
      AnyCalendarKind_Japanese = 13,
      AnyCalendarKind_JapaneseExtended = 14,
      AnyCalendarKind_Persian = 15,
      AnyCalendarKind_Roc = 16,
    };

    typedef struct AnyCalendarKind_option {union { AnyCalendarKind ok; }; bool is_ok; } AnyCalendarKind_option;
} // namespace capi
} // namespace

namespace temporal {
class AnyCalendarKind {
public:
    enum Value {
        Iso = 0,
        Buddhist = 1,
        Chinese = 2,
        Coptic = 3,
        Dangi = 4,
        Ethiopian = 5,
        EthiopianAmeteAlem = 6,
        Gregorian = 7,
        Hebrew = 8,
        Indian = 9,
        HijriTabularTypeIIFriday = 10,
        HijriTabularTypeIIThursday = 11,
        HijriUmmAlQura = 12,
        Japanese = 13,
        JapaneseExtended = 14,
        Persian = 15,
        Roc = 16,
    };

    AnyCalendarKind(): value(Value::Iso) {}

    // Implicit conversions between enum and ::Value
    constexpr AnyCalendarKind(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

  inline static std::optional<temporal::AnyCalendarKind> get_for_str(std::string_view s);

  inline static std::optional<temporal::AnyCalendarKind> parse_temporal_calendar_string(std::string_view s);

    inline temporal::capi::AnyCalendarKind AsFFI() const;
    inline static temporal::AnyCalendarKind FromFFI(temporal::capi::AnyCalendarKind c_enum);
private:
    Value value;
};

} // namespace
#endif // TEMPORAL_AnyCalendarKind_D_HPP
