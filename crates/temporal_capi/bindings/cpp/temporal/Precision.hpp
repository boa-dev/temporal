#ifndef TEMPORAL_Precision_HPP
#define TEMPORAL_Precision_HPP

#include "Precision.d.hpp"

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


inline temporal::capi::Precision temporal::Precision::AsFFI() const {
    return temporal::capi::Precision {
        /* .is_minute = */ is_minute,
        /* .precision = */ precision.has_value() ? (temporal::diplomat::capi::OptionU8{ { precision.value() }, true }) : (temporal::diplomat::capi::OptionU8{ {}, false }),
    };
}

inline temporal::Precision temporal::Precision::FromFFI(temporal::capi::Precision c_struct) {
    return temporal::Precision {
        /* .is_minute = */ c_struct.is_minute,
        /* .precision = */ c_struct.precision.is_ok ? std::optional(c_struct.precision.ok) : std::nullopt,
    };
}


#endif // TEMPORAL_Precision_HPP
