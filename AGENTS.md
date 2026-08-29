# AGENTS.md

## Repository overview
This project is a browser-native digital audio workstation with a React/TypeScript control plane and a Rust/WASM execution plane.

## Important engineering constraints
- Keep DSP logic in Rust/WASM, not JavaScript.
- Do not allocate in the audio render callback.
- Do not add unnecessary abstractions.
- Do not silently change the ABI.
- Prefer deterministic behavior and test coverage for behavioral changes.
- Respect the browser requirements for AudioWorklet, SharedArrayBuffer, and cross-origin isolation.

## AI provider setup
Use OpenRouter as the default external model provider.

Required environment variable:
- OPENROUTER_API_KEY

Kilo config is already set to use OpenRouter via `kilo.json` and reads the key from the environment.

## Local validation commands
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo test --workspace
- cargo build --workspace --target wasm32-unknown-unknown
- npm test
- npm run build

## Working conventions
- Keep changes minimal and consistent with the current architecture.
- Preserve the no_std and lock-free assumptions in the render path.
- Use explicit, well-scoped changes rather than broad refactors.
- When behavior changes, add or update tests.
