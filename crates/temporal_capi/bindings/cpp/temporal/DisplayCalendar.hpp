#ifndef TEMPORAL_DisplayCalendar_HPP
#define TEMPORAL_DisplayCalendar_HPP

#include "DisplayCalendar.d.hpp"

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

} // namespace capi
} // namespace

inline temporal::capi::DisplayCalendar temporal::DisplayCalendar::AsFFI() const {
    return static_cast<temporal::capi::DisplayCalendar>(value);
}

inline temporal::DisplayCalendar temporal::DisplayCalendar::FromFFI(temporal::capi::DisplayCalendar c_enum) {
    switch (c_enum) {
        case temporal::capi::DisplayCalendar_Auto:
        case temporal::capi::DisplayCalendar_Always:
        case temporal::capi::DisplayCalendar_Never:
        case temporal::capi::DisplayCalendar_Critical:
            return static_cast<temporal::DisplayCalendar::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_DisplayCalendar_HPP
