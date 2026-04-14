# Conductor development commands

# Build the project
build:
    cargo build

# Run the app
run:
    cargo run

# Build and run
dev:
    cargo run

# Run tests
test:
    cargo test

# Capture a screenshot of the Conductor window
screencapture:
    #!/usr/bin/env bash
    set -euo pipefail
    WID=$(swift -e '
    import CoreGraphics
    if let ws = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] {
        for w in ws {
            if (w["kCGWindowName"] as? String ?? "") == "Conductor" {
                print(w["kCGWindowNumber"] as? Int ?? 0)
                break
            }
        }
    }
    ' 2>/dev/null | grep -E '^[0-9]+$' || true)
    if [ -z "$WID" ]; then
        echo "error: Conductor window not found. Is the app running?"
        exit 1
    fi
    osascript -e 'tell application "System Events" to tell (first process whose name is "conductor-app") to set frontmost to true' 2>/dev/null || true
    sleep 0.5
    OUT="/tmp/conductor_$(date +%Y%m%d_%H%M%S).png"
    screencapture -x -l "$WID" "$OUT"
    echo "saved: $OUT"
    open "$OUT"
