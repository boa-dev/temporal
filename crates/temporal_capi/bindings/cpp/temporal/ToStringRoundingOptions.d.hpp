#ifndef TEMPORAL_ToStringRoundingOptions_D_HPP
#define TEMPORAL_ToStringRoundingOptions_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Precision.d.hpp"
#include "RoundingMode.d.hpp"
#include "Unit.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
struct Precision;
class RoundingMode;
class Unit;
} // namespace temporal



namespace temporal {
namespace capi {
    struct ToStringRoundingOptions {
      temporal::capi::Precision precision;
      temporal::capi::Unit_option smallest_unit;
      temporal::capi::RoundingMode_option rounding_mode;
    };

    typedef struct ToStringRoundingOptions_option {union { ToStringRoundingOptions ok; }; bool is_ok; } ToStringRoundingOptions_option;
} // namespace capi
} // namespace


namespace temporal {
struct ToStringRoundingOptions {
    temporal::Precision precision;
    std::optional<temporal::Unit> smallest_unit;
    std::optional<temporal::RoundingMode> rounding_mode;

    inline temporal::capi::ToStringRoundingOptions AsFFI() const;
    inline static temporal::ToStringRoundingOptions FromFFI(temporal::capi::ToStringRoundingOptions c_struct);
};

} // namespace
#endif // TEMPORAL_ToStringRoundingOptions_D_HPP
