#!/usr/bin/env bash

awk '
/cargo clippy --fix/ {
    if (match($0, /apply ([0-9]+) suggestions?/, m) &&
        match($0, /(cargo clippy --fix[^)]*)/, c)) {
        print m[1] "\t" c[1]
    }
}
' \
| sort -nr \
| awk -F '\t' '{
    printf "%d. %s  (%s suggestions)\n", NR, $2, $1
}'
