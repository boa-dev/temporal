#ifndef TEMPORAL_RoundingOptions_D_HPP
#define TEMPORAL_RoundingOptions_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "RoundingMode.d.hpp"
#include "Unit.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
class RoundingMode;
class Unit;
} // namespace temporal



namespace temporal {
namespace capi {
    struct RoundingOptions {
      temporal::capi::Unit_option largest_unit;
      temporal::capi::Unit_option smallest_unit;
      temporal::capi::RoundingMode_option rounding_mode;
      temporal::diplomat::capi::OptionU32 increment;
    };

    typedef struct RoundingOptions_option {union { RoundingOptions ok; }; bool is_ok; } RoundingOptions_option;
} // namespace capi
} // namespace


namespace temporal {
struct RoundingOptions {
    std::optional<temporal::Unit> largest_unit;
    std::optional<temporal::Unit> smallest_unit;
    std::optional<temporal::RoundingMode> rounding_mode;
    std::optional<uint32_t> increment;

    inline temporal::capi::RoundingOptions AsFFI() const;
    inline static temporal::RoundingOptions FromFFI(temporal::capi::RoundingOptions c_struct);
};

} // namespace
#endif // TEMPORAL_RoundingOptions_D_HPP
