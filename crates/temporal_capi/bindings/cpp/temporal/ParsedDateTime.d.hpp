#ifndef TEMPORAL_ParsedDateTime_D_HPP
#define TEMPORAL_ParsedDateTime_D_HPP

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
namespace capi { struct ParsedDateTime; }
class ParsedDateTime;
struct TemporalError;
} // namespace temporal



namespace temporal {
namespace capi {
    struct ParsedDateTime;
} // namespace capi
} // namespace

namespace temporal {
class ParsedDateTime {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDateTime>, temporal::TemporalError> from_utf16(std::u16string_view s);

    inline const temporal::capi::ParsedDateTime* AsFFI() const;
    inline temporal::capi::ParsedDateTime* AsFFI();
    inline static const temporal::ParsedDateTime* FromFFI(const temporal::capi::ParsedDateTime* ptr);
    inline static temporal::ParsedDateTime* FromFFI(temporal::capi::ParsedDateTime* ptr);
    inline static void operator delete(void* ptr);
private:
    ParsedDateTime() = delete;
    ParsedDateTime(const temporal::ParsedDateTime&) = delete;
    ParsedDateTime(temporal::ParsedDateTime&&) noexcept = delete;
    ParsedDateTime operator=(const temporal::ParsedDateTime&) = delete;
    ParsedDateTime operator=(temporal::ParsedDateTime&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_ParsedDateTime_D_HPP
