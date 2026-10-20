#ifndef TEMPORAL_DifferenceSettings_D_HPP
#define TEMPORAL_DifferenceSettings_D_HPP

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
    struct DifferenceSettings {
      temporal::capi::Unit_option largest_unit;
      temporal::capi::Unit_option smallest_unit;
      temporal::capi::RoundingMode_option rounding_mode;
      temporal::diplomat::capi::OptionU32 increment;
    };

    typedef struct DifferenceSettings_option {union { DifferenceSettings ok; }; bool is_ok; } DifferenceSettings_option;
} // namespace capi
} // namespace


namespace temporal {
struct DifferenceSettings {
    std::optional<temporal::Unit> largest_unit;
    std::optional<temporal::Unit> smallest_unit;
    std::optional<temporal::RoundingMode> rounding_mode;
    std::optional<uint32_t> increment;

    inline temporal::capi::DifferenceSettings AsFFI() const;
    inline static temporal::DifferenceSettings FromFFI(temporal::capi::DifferenceSettings c_struct);
};

} // namespace
#endif // TEMPORAL_DifferenceSettings_D_HPP
