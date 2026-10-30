#ifndef TEMPORAL_PartialDateTime_D_HPP
#define TEMPORAL_PartialDateTime_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "PartialDate.d.hpp"
#include "PartialTime.d.hpp"
#include "diplomat_runtime.hpp"
namespace temporal {
struct PartialDate;
struct PartialTime;
} // namespace temporal



namespace temporal {
namespace capi {
    struct PartialDateTime {
      temporal::capi::PartialDate date;
      temporal::capi::PartialTime time;
    };

    typedef struct PartialDateTime_option {union { PartialDateTime ok; }; bool is_ok; } PartialDateTime_option;
} // namespace capi
} // namespace


namespace temporal {
struct PartialDateTime {
    temporal::PartialDate date;
    temporal::PartialTime time;

    inline temporal::capi::PartialDateTime AsFFI() const;
    inline static temporal::PartialDateTime FromFFI(temporal::capi::PartialDateTime c_struct);
};

} // namespace
#endif // TEMPORAL_PartialDateTime_D_HPP
