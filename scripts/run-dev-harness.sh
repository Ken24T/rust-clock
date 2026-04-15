#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
debug_binary="$repo_root/target/debug/rust-clock"

find_stale_debug_pids() {
    local proc_dir pid exe_path

    for proc_dir in /proc/[0-9]*; do
        pid=${proc_dir#/proc/}
        exe_path=$(readlink -f "$proc_dir/exe" 2>/dev/null || true)

        if [[ "$exe_path" == "$debug_binary" ]]; then
            printf '%s\n' "$pid"
        fi
    done
}

mapfile -t stale_pids < <(find_stale_debug_pids)

if (( ${#stale_pids[@]} > 0 )); then
    echo "Stopping stale debug harness process(es): ${stale_pids[*]}"
    kill "${stale_pids[@]}"

    for _ in {1..20}; do
        mapfile -t remaining < <(find_stale_debug_pids)

        if (( ${#remaining[@]} == 0 )); then
            break
        fi

        sleep 0.25
    done

    mapfile -t remaining < <(find_stale_debug_pids)

    if (( ${#remaining[@]} > 0 )); then
        echo "Debug harness process(es) did not exit cleanly: ${remaining[*]}" >&2
        echo "Resolve them manually before launching a new review session." >&2
        exit 1
    fi
fi

cd "$repo_root"
exec cargo run "$@"