#ifndef TEMPORAL_TemporalError_HPP
#define TEMPORAL_TemporalError_HPP

#include "TemporalError.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "ErrorKind.hpp"
#include "diplomat_runtime.hpp"


namespace temporal {
namespace capi {

} // namespace capi
} // namespace


inline temporal::capi::TemporalError temporal::TemporalError::AsFFI() const {
    return temporal::capi::TemporalError {
        /* .kind = */ kind.AsFFI(),
        /* .msg = */ msg.has_value() ? (temporal::diplomat::capi::OptionStringView{ { {msg.value().data(), msg.value().size()} }, true }) : (temporal::diplomat::capi::OptionStringView{ {}, false }),
    };
}

inline temporal::TemporalError temporal::TemporalError::FromFFI(temporal::capi::TemporalError c_struct) {
    return temporal::TemporalError {
        /* .kind = */ temporal::ErrorKind::FromFFI(c_struct.kind),
        /* .msg = */ c_struct.msg.is_ok ? std::optional(std::string_view(c_struct.msg.ok.data, c_struct.msg.ok.len)) : std::nullopt,
    };
}


#endif // TEMPORAL_TemporalError_HPP
