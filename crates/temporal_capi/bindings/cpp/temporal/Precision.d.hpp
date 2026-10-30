#ifndef TEMPORAL_Precision_D_HPP
#define TEMPORAL_Precision_D_HPP

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
    struct Precision {
      bool is_minute;
      temporal::diplomat::capi::OptionU8 precision;
    };

    typedef struct Precision_option {union { Precision ok; }; bool is_ok; } Precision_option;
} // namespace capi
} // namespace


namespace temporal {
struct Precision {
    bool is_minute;
    std::optional<uint8_t> precision;

    inline temporal::capi::Precision AsFFI() const;
    inline static temporal::Precision FromFFI(temporal::capi::Precision c_struct);
};

} // namespace
#endif // TEMPORAL_Precision_D_HPP
