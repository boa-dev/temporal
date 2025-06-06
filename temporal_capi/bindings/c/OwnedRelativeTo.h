#ifndef OwnedRelativeTo_H
#define OwnedRelativeTo_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "TemporalError.d.h"

#include "OwnedRelativeTo.d.h"






typedef struct temporal_rs_OwnedRelativeTo_try_from_str_result {union {OwnedRelativeTo ok; TemporalError err;}; bool is_ok;} temporal_rs_OwnedRelativeTo_try_from_str_result;
temporal_rs_OwnedRelativeTo_try_from_str_result temporal_rs_OwnedRelativeTo_try_from_str(DiplomatStringView source);

OwnedRelativeTo temporal_rs_OwnedRelativeTo_empty(void);





#endif // OwnedRelativeTo_H
