#ifndef TEMPORAL_Instant_D_HPP
#define TEMPORAL_Instant_D_HPP

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
namespace capi { struct Duration; }
class Duration;
namespace capi { struct Instant; }
class Instant;
namespace capi { struct Provider; }
class Provider;
namespace capi { struct ZonedDateTime; }
class ZonedDateTime;
struct DifferenceSettings;
struct I128Nanoseconds;
struct RoundingOptions;
struct TemporalError;
struct TimeZone;
struct ToStringRoundingOptions;
} // namespace temporal



namespace temporal {
namespace capi {
    struct Instant;
} // namespace capi
} // namespace

namespace temporal {
class Instant {
public:

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> try_new(temporal::I128Nanoseconds ns);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> from_epoch_milliseconds(int64_t epoch_milliseconds);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> from_utf8(std::string_view s);

  inline static temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> from_utf16(std::u16string_view s);

  inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> add(const temporal::Duration& duration) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> subtract(const temporal::Duration& duration) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> since(const temporal::Instant& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Duration>, temporal::TemporalError> until(const temporal::Instant& other, temporal::DifferenceSettings settings) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::Instant>, temporal::TemporalError> round(temporal::RoundingOptions options) const;

  inline int8_t compare(const temporal::Instant& other) const;

  inline bool equals(const temporal::Instant& other) const;

  inline int64_t epoch_milliseconds() const;

  inline temporal::I128Nanoseconds epoch_nanoseconds() const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string_with_compiled_data(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_with_compiled_data_write(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, W& writeable_output) const;

  inline temporal::diplomat::result<std::string, temporal::TemporalError> to_ixdtf_string_with_provider(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, const temporal::Provider& p) const;
  template<typename W>
  inline temporal::diplomat::result<std::monostate, temporal::TemporalError> to_ixdtf_string_with_provider_write(std::optional<temporal::TimeZone> zone, temporal::ToStringRoundingOptions options, const temporal::Provider& p, W& writeable_output) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time_iso(temporal::TimeZone zone) const;

  inline temporal::diplomat::result<std::unique_ptr<temporal::ZonedDateTime>, temporal::TemporalError> to_zoned_date_time_iso_with_provider(temporal::TimeZone zone, const temporal::Provider& p) const;

  inline std::unique_ptr<temporal::Instant> clone() const;

    inline const temporal::capi::Instant* AsFFI() const;
    inline temporal::capi::Instant* AsFFI();
    inline static const temporal::Instant* FromFFI(const temporal::capi::Instant* ptr);
    inline static temporal::Instant* FromFFI(temporal::capi::Instant* ptr);
    inline static void operator delete(void* ptr);
private:
    Instant() = delete;
    Instant(const temporal::Instant&) = delete;
    Instant(temporal::Instant&&) noexcept = delete;
    Instant operator=(const temporal::Instant&) = delete;
    Instant operator=(temporal::Instant&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // TEMPORAL_Instant_D_HPP
