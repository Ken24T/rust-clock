#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)

binary_source="$repo_root/target/release/rust-clock"
binary_target="$HOME/.local/bin/rust-clock"
applications_dir="$HOME/.local/share/applications"
desktop_source="$repo_root/assets/rust-clock.desktop"
desktop_target="$applications_dir/rust-clock.desktop"
tmp_desktop=$(mktemp)

cleanup() {
    rm -f "$tmp_desktop"
}

trap cleanup EXIT

install -Dm755 "$binary_source" "$binary_target"

awk -v exec_path="$binary_target" '
    BEGIN {
        replaced_exec = 0
    }

    /^TryExec=/ {
        next
    }

    /^Exec=/ {
        print "TryExec=" exec_path
        print "Exec=" exec_path
        replaced_exec = 1
        next
    }

    {
        print
    }

    END {
        if (!replaced_exec) {
            print "TryExec=" exec_path
            print "Exec=" exec_path
        }
    }
' "$desktop_source" > "$tmp_desktop"

install -Dm644 "$tmp_desktop" "$desktop_target"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$applications_dir" >/dev/null 2>&1 || true
fi

echo "Installed binary: $binary_target"
echo "Installed desktop entry: $desktop_target"