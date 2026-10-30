#ifndef TEMPORAL_PartialTime_D_HPP
#define TEMPORAL_PartialTime_D_HPP

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
    struct PartialTime {
      temporal::diplomat::capi::OptionU8 hour;
      temporal::diplomat::capi::OptionU8 minute;
      temporal::diplomat::capi::OptionU8 second;
      temporal::diplomat::capi::OptionU16 millisecond;
      temporal::diplomat::capi::OptionU16 microsecond;
      temporal::diplomat::capi::OptionU16 nanosecond;
    };

    typedef struct PartialTime_option {union { PartialTime ok; }; bool is_ok; } PartialTime_option;
} // namespace capi
} // namespace


namespace temporal {
struct PartialTime {
    std::optional<uint8_t> hour;
    std::optional<uint8_t> minute;
    std::optional<uint8_t> second;
    std::optional<uint16_t> millisecond;
    std::optional<uint16_t> microsecond;
    std::optional<uint16_t> nanosecond;

    inline temporal::capi::PartialTime AsFFI() const;
    inline static temporal::PartialTime FromFFI(temporal::capi::PartialTime c_struct);
};

} // namespace
#endif // TEMPORAL_PartialTime_D_HPP
