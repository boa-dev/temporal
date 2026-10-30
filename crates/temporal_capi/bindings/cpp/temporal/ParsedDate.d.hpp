#ifndef TEMPORAL_ParsedDate_D_HPP
#define TEMPORAL_ParsedDate_D_HPP

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
namespace capi { struct ParsedDate; }
class ParsedDate;
struct TemporalError;
} // namespace temporal



namespace temporal {
namespace capi {
    struct ParsedDate;
} // namespace capi
} // namespace

namespace temporal {
class ParsedDate {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> year_month_from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> year_month_from_utf16(std::u16string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> month_day_from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::ParsedDate>, temporal::TemporalError> month_day_from_utf16(std::u16string_view s);

    inline const temporal::capi::ParsedDate* AsFFI() const;
    inline temporal::capi::ParsedDate* AsFFI();
    inline static const temporal::ParsedDate* FromFFI(const temporal::capi::ParsedDate* ptr);
    inline static temporal::ParsedDate* FromFFI(temporal::capi::ParsedDate* ptr);
    inline static void operator delete(void* ptr);
private:
    ParsedDate() = delete;
    ParsedDate(const temporal::ParsedDate&) = delete;
    ParsedDate(temporal::ParsedDate&&) noexcept = delete;
    ParsedDate operator=(const temporal::ParsedDate&) = delete;
    ParsedDate operator=(temporal::ParsedDate&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_ParsedDate_D_HPP
