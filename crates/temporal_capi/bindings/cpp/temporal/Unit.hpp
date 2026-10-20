#ifndef TEMPORAL_Unit_HPP
#define TEMPORAL_Unit_HPP

#include "Unit.d.hpp"

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

inline temporal::capi::Unit temporal::Unit::AsFFI() const {
    return static_cast<temporal::capi::Unit>(value);
}

inline temporal::Unit temporal::Unit::FromFFI(temporal::capi::Unit c_enum) {
    switch (c_enum) {
        case temporal::capi::Unit_Auto:
        case temporal::capi::Unit_Nanosecond:
        case temporal::capi::Unit_Microsecond:
        case temporal::capi::Unit_Millisecond:
        case temporal::capi::Unit_Second:
        case temporal::capi::Unit_Minute:
        case temporal::capi::Unit_Hour:
        case temporal::capi::Unit_Day:
        case temporal::capi::Unit_Week:
        case temporal::capi::Unit_Month:
        case temporal::capi::Unit_Year:
            return static_cast<temporal::Unit::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_Unit_HPP
