#ifndef TEMPORAL_RelativeTo_D_HPP
#define TEMPORAL_RelativeTo_D_HPP

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
namespace capi { struct PlainDate; }
class PlainDate;
namespace capi { struct ZonedDateTime; }
class ZonedDateTime;
} // namespace temporal



namespace temporal {
namespace capi {
    struct RelativeTo {
      const temporal::capi::PlainDate* date;
      const temporal::capi::ZonedDateTime* zoned;
    };

    typedef struct RelativeTo_option {union { RelativeTo ok; }; bool is_ok; } RelativeTo_option;
} // namespace capi
} // namespace


namespace temporal {
struct RelativeTo {
    const temporal::PlainDate* date;
    const temporal::ZonedDateTime* zoned;

    inline temporal::capi::RelativeTo AsFFI() const;
    inline static temporal::RelativeTo FromFFI(temporal::capi::RelativeTo c_struct);
};

} // namespace
#endif // TEMPORAL_RelativeTo_D_HPP
