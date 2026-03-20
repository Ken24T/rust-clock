#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
debug_binary="$repo_root/target/debug/rust-clock"

find_stale_debug_pids() {
    ps -eo pid=,args= | awk -v binary="$debug_binary" '
        index($0, binary) {
            print $1
        }
    '
}

stale_pids=$(find_stale_debug_pids)

if [[ -n "$stale_pids" ]]; then
    echo "Stopping stale debug harness process(es): $stale_pids"
    kill $stale_pids

    for _ in {1..20}; do
        remaining=$(find_stale_debug_pids)

        if [[ -z "$remaining" ]]; then
            break
        fi

        sleep 0.25
    done

    remaining=$(find_stale_debug_pids)

    if [[ -n "$remaining" ]]; then
        echo "Debug harness process(es) did not exit cleanly: $remaining" >&2
        echo "Resolve them manually before launching a new review session." >&2
        exit 1
    fi
fi

cd "$repo_root"
exec cargo run "$@"