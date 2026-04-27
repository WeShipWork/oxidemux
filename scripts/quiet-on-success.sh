#!/bin/sh

if [ "$#" -eq 0 ]; then
    printf '%s\n' "usage: quiet-on-success.sh command [args...]" >&2
    exit 64
fi

tmp_file=$(mktemp)
trap 'rm -f "$tmp_file"' EXIT HUP INT TERM

"$@" >"$tmp_file" 2>&1
status=$?

if [ "$status" -eq 0 ]; then
    rm -f "$tmp_file"
    trap - EXIT HUP INT TERM
    exit 0
fi

cat "$tmp_file"
rm -f "$tmp_file"
trap - EXIT HUP INT TERM
exit "$status"
