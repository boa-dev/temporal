#ifndef TEMPORAL_OwnedRelativeTo_D_HPP
#define TEMPORAL_OwnedRelativeTo_D_HPP

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
namespace capi { struct Provider; }
class Provider;
namespace capi { struct ZonedDateTime; }
class ZonedDateTime;
struct OwnedRelativeTo;
struct TemporalError;
} // namespace temporal



namespace temporal {
namespace capi {
    struct OwnedRelativeTo {
      temporal::capi::PlainDate* date;
      temporal::capi::ZonedDateTime* zoned;
    };

    typedef struct OwnedRelativeTo_option {union { OwnedRelativeTo ok; }; bool is_ok; } OwnedRelativeTo_option;
} // namespace capi
} // namespace


namespace temporal {
/**
 * GetTemporalRelativeToOption can create fresh PlainDate/ZonedDateTimes by parsing them,
 * we need a way to produce that result.
 */
struct OwnedRelativeTo {
    std::unique_ptr<temporal::PlainDate> date;
    std::unique_ptr<temporal::ZonedDateTime> zoned;

  inline static temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> from_utf8_with_provider(std::string_view s, const temporal::Provider& p);

  inline static temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline static temporal::diplomat::result<temporal::OwnedRelativeTo, temporal::TemporalError> from_utf16_with_provider(std::u16string_view s, const temporal::Provider& p);

  inline static temporal::OwnedRelativeTo empty();

    inline temporal::capi::OwnedRelativeTo AsFFI() const;
    inline static temporal::OwnedRelativeTo FromFFI(temporal::capi::OwnedRelativeTo c_struct);
};

} // namespace
#endif // TEMPORAL_OwnedRelativeTo_D_HPP
