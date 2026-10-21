#ifndef TEMPORAL_DisplayTimeZone_HPP
#define TEMPORAL_DisplayTimeZone_HPP

#include "DisplayTimeZone.d.hpp"

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

inline temporal::capi::DisplayTimeZone temporal::DisplayTimeZone::AsFFI() const {
    return static_cast<temporal::capi::DisplayTimeZone>(value);
}

inline temporal::DisplayTimeZone temporal::DisplayTimeZone::FromFFI(temporal::capi::DisplayTimeZone c_enum) {
    switch (c_enum) {
        case temporal::capi::DisplayTimeZone_Auto:
        case temporal::capi::DisplayTimeZone_Never:
        case temporal::capi::DisplayTimeZone_Critical:
            return static_cast<temporal::DisplayTimeZone::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_DisplayTimeZone_HPP
