#ifndef TEMPORAL_ToStringRoundingOptions_HPP
#define TEMPORAL_ToStringRoundingOptions_HPP

#include "ToStringRoundingOptions.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Precision.hpp"
#include "RoundingMode.hpp"
#include "Unit.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::ToStringRoundingOptions temporal::ToStringRoundingOptions::AsFFI() const {
    return temporal::capi::ToStringRoundingOptions {
        /* .precision = */ precision.AsFFI(),
        /* .smallest_unit = */ smallest_unit.has_value() ? (temporal::capi::Unit_option{ { smallest_unit.value().AsFFI() }, true }) : (temporal::capi::Unit_option{ {}, false }),
        /* .rounding_mode = */ rounding_mode.has_value() ? (temporal::capi::RoundingMode_option{ { rounding_mode.value().AsFFI() }, true }) : (temporal::capi::RoundingMode_option{ {}, false }),
    };
}

inline temporal::ToStringRoundingOptions temporal::ToStringRoundingOptions::FromFFI(temporal::capi::ToStringRoundingOptions c_struct) {
    return temporal::ToStringRoundingOptions {
        /* .precision = */ temporal::Precision::FromFFI(c_struct.precision),
        /* .smallest_unit = */ c_struct.smallest_unit.is_ok ? std::optional(temporal::Unit::FromFFI(c_struct.smallest_unit.ok)) : std::nullopt,
        /* .rounding_mode = */ c_struct.rounding_mode.is_ok ? std::optional(temporal::RoundingMode::FromFFI(c_struct.rounding_mode.ok)) : std::nullopt,
    };
}


#endif // TEMPORAL_ToStringRoundingOptions_HPP
