#ifndef TEMPORAL_AnyCalendarKind_HPP
#define TEMPORAL_AnyCalendarKind_HPP

#include "AnyCalendarKind.d.hpp"

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
namespace capi {
    extern "C" {

    typedef struct temporal_AnyCalendarKind_get_for_str_result {union {temporal::capi::AnyCalendarKind ok; }; bool is_ok;} temporal_AnyCalendarKind_get_for_str_result;
    temporal_AnyCalendarKind_get_for_str_result temporal_AnyCalendarKind_get_for_str(temporal::diplomat::capi::DiplomatStringView s);

    typedef struct temporal_AnyCalendarKind_parse_temporal_calendar_string_result {union {temporal::capi::AnyCalendarKind ok; }; bool is_ok;} temporal_AnyCalendarKind_parse_temporal_calendar_string_result;
    temporal_AnyCalendarKind_parse_temporal_calendar_string_result temporal_AnyCalendarKind_parse_temporal_calendar_string(temporal::diplomat::capi::DiplomatStringView s);

    } // extern "C"
} // namespace capi
} // namespace

inline temporal::capi::AnyCalendarKind temporal::AnyCalendarKind::AsFFI() const {
    return static_cast<temporal::capi::AnyCalendarKind>(value);
}

inline temporal::AnyCalendarKind temporal::AnyCalendarKind::FromFFI(temporal::capi::AnyCalendarKind c_enum) {
    switch (c_enum) {
        case temporal::capi::AnyCalendarKind_Iso:
        case temporal::capi::AnyCalendarKind_Buddhist:
        case temporal::capi::AnyCalendarKind_Chinese:
        case temporal::capi::AnyCalendarKind_Coptic:
        case temporal::capi::AnyCalendarKind_Dangi:
        case temporal::capi::AnyCalendarKind_Ethiopian:
        case temporal::capi::AnyCalendarKind_EthiopianAmeteAlem:
        case temporal::capi::AnyCalendarKind_Gregorian:
        case temporal::capi::AnyCalendarKind_Hebrew:
        case temporal::capi::AnyCalendarKind_Indian:
        case temporal::capi::AnyCalendarKind_HijriTabularTypeIIFriday:
        case temporal::capi::AnyCalendarKind_HijriTabularTypeIIThursday:
        case temporal::capi::AnyCalendarKind_HijriUmmAlQura:
        case temporal::capi::AnyCalendarKind_Japanese:
        case temporal::capi::AnyCalendarKind_JapaneseExtended:
        case temporal::capi::AnyCalendarKind_Persian:
        case temporal::capi::AnyCalendarKind_Roc:
            return static_cast<temporal::AnyCalendarKind::Value>(c_enum);
        default:
            std::abort();
    }
}

inline std::optional<temporal::AnyCalendarKind> temporal::AnyCalendarKind::get_for_str(std::string_view s) {
    auto result = temporal::capi::temporal_AnyCalendarKind_get_for_str({s.data(), s.size()});
    return result.is_ok ? std::optional<temporal::AnyCalendarKind>(temporal::AnyCalendarKind::FromFFI(result.ok)) : std::nullopt;
}

inline std::optional<temporal::AnyCalendarKind> temporal::AnyCalendarKind::parse_temporal_calendar_string(std::string_view s) {
    auto result = temporal::capi::temporal_AnyCalendarKind_parse_temporal_calendar_string({s.data(), s.size()});
    return result.is_ok ? std::optional<temporal::AnyCalendarKind>(temporal::AnyCalendarKind::FromFFI(result.ok)) : std::nullopt;
}
#endif // TEMPORAL_AnyCalendarKind_HPP
