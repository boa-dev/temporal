#ifndef TEMPORAL_TemporalError_D_HPP
#define TEMPORAL_TemporalError_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "ErrorKind.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
class ErrorKind;
} // namespace temporal



namespace temporal {
namespace capi {
    struct TemporalError {
      temporal::capi::ErrorKind kind;
      temporal::diplomat::capi::OptionStringView msg;
    };

    typedef struct TemporalError_option {union { TemporalError ok; }; bool is_ok; } TemporalError_option;
} // namespace capi
} // namespace


namespace temporal {
struct TemporalError {
    temporal::ErrorKind kind;
    std::optional<std::string_view> msg;

    inline temporal::capi::TemporalError AsFFI() const;
    inline static temporal::TemporalError FromFFI(temporal::capi::TemporalError c_struct);
};

} // namespace
#endif // TEMPORAL_TemporalError_D_HPP
