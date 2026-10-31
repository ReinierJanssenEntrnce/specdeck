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
