#!/usr/bin/env bash


# I used this when I was running cargo clippy --fix for the first time on 258 warnings. 
# I suppose if cargo clippy is used as often as, or instead of, check, this script
# is not all that useful since all clippy warnings are either disabled or otherwise addressed immediately. 
# I have not used it ever since that time. Consider removing it?

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
