set windows-shell := ["pwsh", "-NoLogo", "-Command"]

# List recipes
default:
    @just --list

# Install dependencies
install:
    pnpm install

# Start the desktop app in development mode
dev:
    pnpm tauri dev

# Check formatting, lint, type-check and test everything
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    pnpm lint
    pnpm typecheck
    pnpm test

# Format the Rust code
fmt:
    cargo fmt --all

# Build installers for this platform (target/release/bundle)
package:
    pnpm tauri build

# Start the documentation site with live reload (http://localhost:4321/specdeck/)
docs:
    cd docs/site && pnpm install && pnpm dev

# Build the documentation site (docs/site/dist)
docs-build:
    cd docs/site && pnpm install --frozen-lockfile && pnpm build

# Deploy signalling and coturn to Colima's local Kubernetes (deploy/k8s/local); needs `colima start --kubernetes`
local-up:
    #!/usr/bin/env bash
    set -euo pipefail
    test -f deploy/k8s/base/turn-secret.env || (umask 077 && echo "TURN_SECRET=$(openssl rand -hex 32)" > deploy/k8s/base/turn-secret.env)
    docker build -f docker/signalling.Dockerfile -t specdeck-signalling:local .
    scripts/local-kubectl.sh apply -k deploy/k8s/local
    # The image tag does not change between builds: restart to pick up the new image.
    scripts/local-kubectl.sh -n specdeck1-local rollout restart deployment/specdeck-signalling
    scripts/local-kubectl.sh -n specdeck1-local rollout status deployment/specdeck-signalling deployment/specdeck-turn --timeout=120s

# Remove the local Kubernetes deployment
local-down:
    scripts/local-kubectl.sh delete -k deploy/k8s/local --ignore-not-found

# Check that two peers connect through the local coturn with direct connections disabled
local-test: local-up
    #!/usr/bin/env bash
    set -euo pipefail
    scripts/local-kubectl.sh -n specdeck1-local port-forward svc/specdeck-signalling 18081:8080 >/dev/null &
    trap 'kill $! 2>/dev/null || true' EXIT
    until curl -sf http://127.0.0.1:18081/health >/dev/null; do sleep 0.25; done
    SPECDECK_SIGNALLING_URL=ws://127.0.0.1:18081/signal cargo test -p specdeck-core --test turn_relay -- --ignored --nocapture

# Local test setup: signalling and coturn on Colima's k3s, plus a host app and a viewer app (own settings each)
local-dev: local-up
    #!/usr/bin/env bash
    set -euo pipefail
    # Each background job gets its own process group, so Ctrl+C stops the whole tree under it
    # (tauri, vite, the apps), not just the job's first process. Background jobs then no longer
    # get /dev/null as input by themselves, so each is given it: one that reads the terminal stops.
    set -m
    stop() { trap - EXIT INT TERM; for pgid in $(jobs -p); do kill -TERM -- -"$pgid" 2>/dev/null || true; done; }
    trap stop EXIT
    trap 'exit 130' INT TERM
    scripts/local-kubectl.sh -n specdeck1-local port-forward svc/specdeck-signalling 18081:8080 >/dev/null </dev/null &
    until curl -sf http://127.0.0.1:18081/health >/dev/null; do sleep 0.25; done
    export SPECDECK_SIGNALLING_URL=ws://127.0.0.1:18081/signal
    mkdir -p .local-dev
    SPECDECK_PROFILE=host pnpm tauri dev </dev/null 2>&1 | tee .local-dev/host.log &
    # The viewer is a second instance of the same debug build, loading the host's dev server.
    until curl -sf http://localhost:5173 >/dev/null && pgrep -f target/debug/specdeck-app >/dev/null; do sleep 0.5; done
    SPECDECK_PROFILE=viewer target/debug/specdeck-app </dev/null 2>&1 | tee .local-dev/viewer.log &
    echo "SpecDeck (host) and SpecDeck (viewer) are starting. Close this with Ctrl+C."
    wait
