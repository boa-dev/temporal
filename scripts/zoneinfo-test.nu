#!/usr/bin/env nu

# Generate JSON ground truth from IANA TZif and run the zoneinfo integration tests.

let repo = $env.PWD
let iana_tzif = $"($repo)/.tzdata/zoneinfo"
let data_out = "../../zoneinfo/tests/data"

def gen [zone: string] {
    cargo run -q -p zoneinfo-test-gen -- -i $iana_tzif -o $data_out $zone
}

# All canonical zones from zone.tab
let all_zones = (
    open $"($repo)/.tzdata/src/zone.tab"
    | lines
    | where not ($it | str starts-with "#") and ($it | str length) > 0
    | each { |l| $l | split row "\t" | get 2 }
)

# Regression zones may include Link zones not in zone.tab — generate those too.
let config = open zoneinfo.toml
let all_zones = ($all_zones ++ $config.regression.zones | uniq)

for zone in $all_zones {
    gen $zone
}
with-env { TZDATA_DIR: $"($repo)/.tzdata/src" } {
    cargo test -p zoneinfo_rs
}

# TODO: rearguard test for Europe/Dublin (ziguard.awk -v DATAFORM=rearguard)
