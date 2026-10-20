#ifndef TEMPORAL_OffsetDisambiguation_HPP
#define TEMPORAL_OffsetDisambiguation_HPP

#include "OffsetDisambiguation.d.hpp"

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

inline temporal::capi::OffsetDisambiguation temporal::OffsetDisambiguation::AsFFI() const {
    return static_cast<temporal::capi::OffsetDisambiguation>(value);
}

inline temporal::OffsetDisambiguation temporal::OffsetDisambiguation::FromFFI(temporal::capi::OffsetDisambiguation c_enum) {
    switch (c_enum) {
        case temporal::capi::OffsetDisambiguation_Use:
        case temporal::capi::OffsetDisambiguation_Prefer:
        case temporal::capi::OffsetDisambiguation_Ignore:
        case temporal::capi::OffsetDisambiguation_Reject:
            return static_cast<temporal::OffsetDisambiguation::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_OffsetDisambiguation_HPP
