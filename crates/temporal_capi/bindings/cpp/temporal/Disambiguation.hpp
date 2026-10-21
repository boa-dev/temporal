#ifndef TEMPORAL_Disambiguation_HPP
#define TEMPORAL_Disambiguation_HPP

#include "Disambiguation.d.hpp"

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

inline temporal::capi::Disambiguation temporal::Disambiguation::AsFFI() const {
    return static_cast<temporal::capi::Disambiguation>(value);
}

inline temporal::Disambiguation temporal::Disambiguation::FromFFI(temporal::capi::Disambiguation c_enum) {
    switch (c_enum) {
        case temporal::capi::Disambiguation_Compatible:
        case temporal::capi::Disambiguation_Earlier:
        case temporal::capi::Disambiguation_Later:
        case temporal::capi::Disambiguation_Reject:
            return static_cast<temporal::Disambiguation::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TEMPORAL_Disambiguation_HPP
