#!/usr/bin/env nu

# Download the pinned IANA tzdata release, verify its checksum, and extract it
# into .tzdata/src/. Also builds rearguard.zi, which the bakeddata tool needs.

let repo = $env.PWD
let config = (open zoneinfo.toml).tzdata
let src = $"($repo)/.tzdata/src"

# Already extracted at the pinned version with rearguard built: nothing to do.
if (($"($src)/version" | path exists) and ($"($src)/rearguard.zi" | path exists)) {
    if (open $"($src)/version" | str trim) == $config.version {
        print $"tzdata ($config.version) already present"
        exit 0
    }
}

let url = $"https://data.iana.org/time-zones/releases/tzdata($config.version).tar.gz"
let tarball = $"($repo)/.tzdata/tzdata($config.version).tar.gz"

mkdir $"($repo)/.tzdata"
print $"Downloading ($url)"
http get --raw $url | save -f -r $tarball

let expected_sha512 = $config.sha512
let actual_sha512 = (^sha512sum $tarball | split row ' ' | get 0)

if $actual_sha512 != $expected_sha512 {
    print $"checksum mismatch for tzdata($config.version)"
    print $"  expected: ($expected_sha512)"
    print $"  actual:   ($actual_sha512)"
    exit 1
}
print $"checksum OK \(sha512\)"

rm -rf $src
mkdir $src
^tar xzf $tarball -C $src

# bakeddata reads rearguard.zi; the tarball ships vanguard by default.
^make -C $src rearguard.zi

print $"tzdata ($config.version) ready at ($src)"
