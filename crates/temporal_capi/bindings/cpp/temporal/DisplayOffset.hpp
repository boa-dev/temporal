#ifndef TEMPORAL_DisplayOffset_HPP
#define TEMPORAL_DisplayOffset_HPP

#include "DisplayOffset.d.hpp"

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

inline temporal::capi::DisplayOffset temporal::DisplayOffset::AsFFI() const {
    return static_cast<temporal::capi::DisplayOffset>(value);
}

inline temporal::DisplayOffset temporal::DisplayOffset::FromFFI(temporal::capi::DisplayOffset c_enum) {
    switch (c_enum) {
        case temporal::capi::DisplayOffset_Auto:
        case temporal::capi::DisplayOffset_Never:
            return static_cast<temporal::DisplayOffset::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_DisplayOffset_HPP
