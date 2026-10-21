#ifndef TEMPORAL_RoundingOptions_HPP
#define TEMPORAL_RoundingOptions_HPP

#include "RoundingOptions.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "RoundingMode.hpp"
#include "Unit.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::RoundingOptions temporal::RoundingOptions::AsFFI() const {
    return temporal::capi::RoundingOptions {
        /* .largest_unit = */ largest_unit.has_value() ? (temporal::capi::Unit_option{ { largest_unit.value().AsFFI() }, true }) : (temporal::capi::Unit_option{ {}, false }),
        /* .smallest_unit = */ smallest_unit.has_value() ? (temporal::capi::Unit_option{ { smallest_unit.value().AsFFI() }, true }) : (temporal::capi::Unit_option{ {}, false }),
        /* .rounding_mode = */ rounding_mode.has_value() ? (temporal::capi::RoundingMode_option{ { rounding_mode.value().AsFFI() }, true }) : (temporal::capi::RoundingMode_option{ {}, false }),
        /* .increment = */ increment.has_value() ? (temporal::diplomat::capi::OptionU32{ { increment.value() }, true }) : (temporal::diplomat::capi::OptionU32{ {}, false }),
    };
}

inline temporal::RoundingOptions temporal::RoundingOptions::FromFFI(temporal::capi::RoundingOptions c_struct) {
    return temporal::RoundingOptions {
        /* .largest_unit = */ c_struct.largest_unit.is_ok ? std::optional(temporal::Unit::FromFFI(c_struct.largest_unit.ok)) : std::nullopt,
        /* .smallest_unit = */ c_struct.smallest_unit.is_ok ? std::optional(temporal::Unit::FromFFI(c_struct.smallest_unit.ok)) : std::nullopt,
        /* .rounding_mode = */ c_struct.rounding_mode.is_ok ? std::optional(temporal::RoundingMode::FromFFI(c_struct.rounding_mode.ok)) : std::nullopt,
        /* .increment = */ c_struct.increment.is_ok ? std::optional(c_struct.increment.ok) : std::nullopt,
    };
}


#endif // TEMPORAL_RoundingOptions_HPP
