# DAW

Browser-native digital audio workstation.

API key managed via `.env` file (gitignored), referenced as
`OPENROUTER_API_KEY` environment variable.

PROJECT
Browser-native digital audio workstation.

ARCHITECTURE
React/TypeScript control plane
        ↓
SharedArrayBuffer
        ↓
AudioWorklet
        ↓
Rust/WASM execution plane
        ↓
Web Audio

RUST REQUIREMENTS
- no_std-compatible kernel
- no allocation in render path
- no blocking
- deterministic execution
- explicit ABI
- lock-free communication

BROWSER REQUIREMENTS
- AudioWorklet-safe execution
- SharedArrayBuffer
- cross-origin isolation
- deterministic render testing

DESIGN RULES
- Don't introduce unnecessary abstractions.
- Don't move DSP logic into JavaScript.
- Don't allocate in the audio render callback.
- Don't silently change the ABI.
- Tests are required for behavioral changes.

VALIDATION
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown
npm test
npm run build
