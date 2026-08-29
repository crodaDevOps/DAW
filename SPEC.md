# SPEC.md — Browser DAW Specification

## Phase 0 — Deterministic Rust/WASM Audio Kernel Foundation
Status: COMPLETE (2026-08-29)
Objective: Establish no_std-compatible DSP kernel executable natively for headless tests and as wasm32-unknown-unknown for browser/AudioWorklet.

### Phase 0 Definition of Done
- [x] Cargo workspace builds
- [x] Rust kernel compiles as no_std (wasm32)
- [x] Native deterministic tests pass
- [x] SPSC transport tests pass
- [x] Transaction framing deterministic
- [x] 128-frame render quantum tested
- [x] 808 glide test passes (boundary <0.0001, 32Hz ±0.5Hz, 96000 frames)
- [x] WASM target installed
- [x] WASM build succeeds (cdylib + rlib)
- [x] Clippy passes (-D warnings)
- [x] rustfmt passes
- [x] No heap allocation in kernel render path (RT-01)
- [x] Browser-facing ABI explicitly represented (RT-06, 24B Mock / 32B Telemetry)

### Phase 0 Scope (implemented)
- `crates/audio-kernel/src/{lib,abi,spsc,kernel,offline}.rs`
- `AudioKernel` trait with `process(&mut [&mut [f32]])`, `apply_transaction`, `read_telemetry`
- `ByteRingBuffer<CAP>` SPSC with [u32 LE len][payload] framing, Acquire/Release
- `DawKernel` oscillator with libm powf/sinf, f64 phase accumulator, exponential glide
- `OfflineDriver` render + quantum-partition invariance
- `libm` deterministic math, `TelemetrySnapshot`/`MockTransaction` #[repr(C)]

### Phase 0 Invariants (RT-01..RT-07)
RT-01 no allocation · RT-02 no blocking · RT-03 deterministic state · RT-04 quantum independence · RT-05 transaction atomicity · RT-06 explicit ABI · RT-07 wasm target

### Phase 0 Out of Scope
AudioWorklet, SharedArrayBuffer, WASM bindings, React UI, browser audio, FlatBuffers, multi-voice, mixer, effects, persistence

### Phase 0 Validation
```
cargo fmt --all -- --check          -> 0
cargo clippy -- -D warnings         -> 0
cargo test --workspace              -> 6/6 pass
cargo build --target wasm32-unknown-unknown -> 0
```

---

## Phase 1 — WASM / AudioWorklet / SharedArrayBuffer Execution Boundary
Status: READY FOR IMPLEMENTATION (precondition Phase 0 complete)
Method: TDD (RED → GREEN → REFACTOR → full validation)
Rule: No Phase 2+ work; Phase 0 invariants must remain green (F1 is STOP).

### Phase 1 Objective
Prove deterministic Phase 0 kernel executes inside browser real-time environment independent of React/main-thread timing:
Rust AudioKernel → wasm32 ABI → TS loader → AudioWorkletProcessor → audio output
Main Thread → SharedArrayBuffer → SPSC → AudioWorklet → AudioKernel

### Phase 1 In Scope (12 items)
1. Explicit WASM ABI exports (`audio_kernel_create/destroy/process/apply_transaction/read_telemetry`)
2. TypeScript WASM loader (`web/audio/wasm-loader.ts`)
3. AudioWorklet module (`web/audio/audio-kernel-processor.ts`)
4. AudioWorkletProcessor
5. AudioWorkletNode integration
6. SharedArrayBuffer transport (write/read indices + byte storage, compatible with Phase 0 framing)
7. SPSC command transport (complete boundaries, no partial visibility)
8. Sample-stamped command execution (targetSample, future pending, ascending order, same-sample sequence ordering, intra-quantum split)
9. Telemetry transport (32B TelemetrySnapshot via shared memory, no alloc in render)
10. Browser integration tests (non-zero output, crossOriginIsolated==true, COOP/COEP)
11. Main-thread starvation test (500ms block, monotonic counter, no reordering)
12. Real-time allocation audit + 500 cmds/sec × 30s stress test (120Hz main load)

### Phase 1 Key Spec Details
- ABI: explicit primitives (ptr/len usize, sample u64, frame usize, float f32); little-endian; do not expose Rust struct layout
- Transaction wire: offset 0: u64 target_sample, 8: f32 target_freq, 12: reserved, 16: u64 duration (24B total, LE)
- Initialization via processorOptions (wasm module, sample rate, SAB refs)
- Render quantum normally 128 but must support arbitrary 1..N; split quantum at sample boundary
- Critical determinism test: targetSample 48000/32Hz, frame 47999 old vs 48000 new, must match offline result
- Telemetry 32B unchanged; no JSON/Object/Vec/String/Promise/async in render path

### Phase 1 File Structure (minimum)
```
browser-daw/
├── crates/audio-kernel/src/{lib,abi,kernel,offline,spsc}.rs
├── web/audio/{wasm-loader.ts,audio-kernel-processor.ts,audio-kernel-node.ts,sab-transport.ts,telemetry.ts,protocol.ts}
├── tests/{wasm,browser,integration}/
└── LOG.md
```

### Phase 1 Validation
```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --target wasm32-unknown-unknown
npm test
npm run build
browser integration tests
```

### Phase 1 Acceptance Matrix (27. matrix)
Phase 0 tests, fmt, clippy, native/wasm build, WASM exports/loader, Worklet init, WASM in Worklet, audio output, SAB, crossOriginIsolated, SPSC, boundaries, sample-stamped/pending/ordering, exact sample, quantum determinism, telemetry, main-thread independence, stress test, no render alloc, no Phase 2 features — all PASS

### Phase 1 Out of Scope
React UI, Zustand, Timeline, Canvas/WebGL/WebGPU, OPFS/IndexedDB, audio file loading/recording, MIDI, multi-voice, mixer, effects, synth expansion, Gemini/AI, Tauri/cpal/native audio, plugin system

### Failure Classification
F1 Phase 0 regression → STOP · F2 ABI → fix ABI · F3 env (SAB/WASM/Worklet/crossOriginIsolated) → report, no fallback · F4 sync → fix sync · F5 determinism → fix boundary · F6 real-time (alloc/block/growth) → STOP · F7 scope creep → STOP

### Completion Criteria
Phase 0 green AND explicit WASM ABI AND WASM loads AND Worklet executes kernel AND SAB functions AND sample-stamped deterministic AND offline==browser AND starvation-tolerant AND stress pass AND allocation contract satisfied
