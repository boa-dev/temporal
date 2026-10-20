#ifndef TEMPORAL_ParsedZonedDateTime_D_HPP
#define TEMPORAL_ParsedZonedDateTime_D_HPP

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
namespace capi { struct ParsedZonedDateTime; }
class ParsedZonedDateTime;
namespace capi { struct Provider; }
class Provider;
struct TemporalError;
} // namespace temporal



namespace temporal {
namespace capi {
    struct ParsedZonedDateTime;
} // namespace capi
} // namespace

namespace temporal {
class ParsedZonedDateTime {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> from_utf8_with_provider(std::string_view s, const temporal::Provider& p);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedZonedDateTime>, temporal::TemporalError> from_utf16_with_provider(std::u16string_view s, const temporal::Provider& p);

    inline const temporal::capi::ParsedZonedDateTime* AsFFI() const;
    inline temporal::capi::ParsedZonedDateTime* AsFFI();
    inline static const temporal::ParsedZonedDateTime* FromFFI(const temporal::capi::ParsedZonedDateTime* ptr);
    inline static temporal::ParsedZonedDateTime* FromFFI(temporal::capi::ParsedZonedDateTime* ptr);
    inline static void operator delete(void* ptr);
private:
    ParsedZonedDateTime() = delete;
    ParsedZonedDateTime(const temporal::ParsedZonedDateTime&) = delete;
    ParsedZonedDateTime(temporal::ParsedZonedDateTime&&) noexcept = delete;
    ParsedZonedDateTime operator=(const temporal::ParsedZonedDateTime&) = delete;
    ParsedZonedDateTime operator=(temporal::ParsedZonedDateTime&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_ParsedZonedDateTime_D_HPP
