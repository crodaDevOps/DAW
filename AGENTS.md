https://github.com/sickn33/antigravity-awesome-skills/tree/main/skills/systems-programming-rust-project


Browser DAW — Phase 1 TDD Implementation Specification

Phase: 1
Title: WASM / AudioWorklet / SharedArrayBuffer Execution Boundary
Status: READY FOR IMPLEMENTATION
Precondition: Phase 0 COMPLETE
Implementation agent: Kilo
Method: Test-Driven Development
Rule: No Phase 2+ work may be introduced.

⸻

1. Purpose

Phase 1 establishes the first real browser execution boundary for the deterministic Phase 0 AudioKernel.

The completed system must demonstrate:

Rust AudioKernel
      ↓
wasm32 WASM ABI
      ↓
TypeScript WASM loader
      ↓
AudioWorkletProcessor
      ↓
Audio output

and then:

Main Thread
      ↓
SharedArrayBuffer
      ↓
SPSC command transport
      ↓
AudioWorklet
      ↓
AudioKernel

The objective is not to build a DAW UI.

The objective is to prove that the deterministic audio kernel can execute inside the browser’s real-time audio environment while remaining independent of React and main-thread frame timing.

⸻

2. Phase 0 Baseline

Phase 0 is considered complete.

Known-good baseline:

cargo fmt --all -- --check
        PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings
        PASS
cargo test --workspace
        PASS — 6/6
cargo build --workspace --target wasm32-unknown-unknown
        PASS

Existing kernel components:

crates/audio-kernel/
├── src/
│   ├── lib.rs
│   ├── abi.rs
│   ├── kernel.rs
│   ├── offline.rs
│   └── spsc.rs
└── Cargo.toml

Phase 1 must preserve all Phase 0 tests.

A Phase 1 implementation that breaks a Phase 0 invariant is a failure.

⸻

3. Scope

IN SCOPE

Phase 1 implements:

1. Explicit WASM ABI exports.
2. TypeScript WASM loading.
3. AudioWorklet module.
4. AudioWorkletProcessor.
5. AudioWorkletNode integration.
6. SharedArrayBuffer transport.
7. SPSC command transport.
8. Sample-stamped command execution.
9. Telemetry transport.
10. Browser integration tests.
11. Main-thread starvation test.
12. Real-time allocation audit.

OUT OF SCOPE

Do NOT implement:

React UI
Zustand project model
Timeline
Canvas
WebGL
WebGPU
OPFS
IndexedDB
Audio file loading
Audio recording
MIDI
Multiple voices
Mixer
Effects
Synthesizer expansion
Gemini
AI commands
Tauri
cpal
Native audio
Plugin system

If implementation requires one of these, stop and report the dependency rather than expanding scope.

⸻

4. Phase 1 Architecture

The implementation must produce this topology:

                    MAIN THREAD
┌─────────────────────────────────────────────┐
│                                             │
│ TypeScript                                  │
│                                             │
│ WASM Loader                                 │
│ AudioWorkletNode                            │
│ SAB Producer                                │
│ Telemetry Reader                            │
│                                             │
└──────────────────────┬──────────────────────┘
                       │
                       │ MessagePort
                       │ initialization only
                       ▼
┌─────────────────────────────────────────────┐
│               AUDIO WORKLET                 │
│                                             │
│ AudioWorkletProcessor                       │
│                                             │
│ SAB Consumer                                │
│ Sample-Stamped Command Queue                │
│                                             │
│              ┌──────────────┐               │
│              │ WASM Kernel  │               │
│              └──────────────┘               │
│                                             │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
                 Audio Output

The main thread must not own the audio clock.

The AudioWorklet must not depend on React.

The WASM kernel must not depend on JavaScript.

⸻

5. TDD Rule

For every implementation feature:

RED
 ↓
write failing test
 ↓
GREEN
 ↓
minimal implementation
 ↓
REFACTOR
 ↓
run complete validation suite

Do not implement an entire phase and test afterward.

Each subsection below defines its acceptance test before its implementation.

⸻

6. Phase 1A — Explicit WASM ABI

Objective

Expose the Phase 0 kernel through an explicit C-compatible ABI.

Do NOT expose Rust struct memory directly as the JavaScript protocol.

⸻

6.1 Required exports

The WASM module must expose functions equivalent to:

audio_kernel_create
audio_kernel_destroy
audio_kernel_process
audio_kernel_apply_transaction
audio_kernel_read_telemetry

Exact naming may vary only if documented consistently.

⸻

6.2 ABI requirements

The ABI must use explicit primitive types.

Required concepts:

pointer → usize
length  → usize
sample  → u64
frame   → usize
float   → f32

No JavaScript-facing function may require JavaScript to understand Rust’s internal struct layout.

⸻

6.3 Transaction wire format

The browser transaction representation must be explicitly defined.

Phase 1 test transaction:

MockTransaction
offset  size  meaning
0       8     target_sample : u64
8       4     target_freq   : f32
12      4     reserved
16      8     duration      : u64

Encoding:

little-endian

The browser protocol must not depend on copying a Rust #[repr(C)] struct directly.

⸻

6.4 Required tests

Create ABI tests that establish:

export exists
export callable
invalid pointer/length is rejected safely
valid transaction is accepted
telemetry can be read

Expected:

PASS

⸻

7. Phase 1B — WASM TypeScript Loader

Create the smallest possible browser-side WASM loader.

Suggested location:

web/audio/wasm-loader.ts

The loader must:

1. Fetch/load the WASM binary.
2. Instantiate it.
3. Resolve required exports.
4. Reject initialization if a required export is missing.
5. Return a typed interface.

Conceptual interface:

interface AudioKernelWasm {
  memory: WebAssembly.Memory;
  audio_kernel_create: (...args: number[]) => number;
  audio_kernel_destroy: (...args: number[]) => void;
  audio_kernel_process: (...args: number[]) => void;
  audio_kernel_apply_transaction: (...args: number[]) => void;
  audio_kernel_read_telemetry: (...args: number[]) => void;
}

The exact generated TypeScript representation may differ.

⸻

Required test

The test must prove:

WASM binary
    ↓
instantiate
    ↓
required exports resolved
    ↓
kernel handle created

Expected result:

PASS

⸻

8. Phase 1C — AudioWorklet

Create:

web/audio/audio-kernel-processor.ts

The processor must extend:

AudioWorkletProcessor

It must own the execution-side kernel state.

The main thread must not directly execute the render loop.

⸻

9. AudioWorklet Initialization Contract

Initialization occurs through processorOptions.

Required information:

WASM binary / module
sample rate
SAB references
ring-buffer configuration

The initialization process must be deterministic.

After initialization:

initialized = true

must be observable through telemetry or a defined initialization response.

⸻

10. First Browser Audio Test

The processor must generate the existing Phase 0 oscillator.

No new synthesizer implementation is permitted.

The test should verify:

AudioContext
      ↓
AudioWorklet
      ↓
WASM
      ↓
AudioKernel
      ↓
non-zero output

The browser test must confirm that the output contains non-zero audio samples.

A test that merely confirms that the processor exists is insufficient.

⸻

11. Phase 1D — SharedArrayBuffer

Create a SharedArrayBuffer for control transport.

Minimum structure:

SharedArrayBuffer
├── write index
├── read index
└── byte storage

The SAB must be compatible with the Phase 0 SPSC framing model.

The exact SAB layout must be documented.

⸻

12. Cross-Origin Isolation

Because SAB requires an isolated execution environment, the development server must provide:

Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp

The browser integration test must explicitly verify:

crossOriginIsolated === true

If false:

FAIL

Do not silently fall back to ordinary ArrayBuffer.

Phase 1 requires SAB.

⸻

13. SAB Command Protocol

The producer runs on the main thread.

The consumer runs inside the AudioWorklet.

Main Thread
    │
    │ write
    ▼
SPSC SAB
    │
    │ read
    ▼
AudioWorklet

The queue must maintain complete transaction boundaries.

A partial transaction must never be visible to the consumer.

⸻

14. Sample-Stamped Commands

Every command affecting future audio state must contain:

targetSample

The AudioWorklet must not execute future commands merely because they have arrived.

Example:

currentSample = 47000
command.targetSample = 48000

The command remains pending.

When:

currentSample >= 48000

the command becomes executable.

⸻

15. Ordering Requirement

Commands must execute in ascending target-sample order.

Given:

A → targetSample 48000
B → targetSample 49000
C → targetSample 50000

execution order must be:

A
B
C

regardless of their arrival timing, provided they were successfully submitted before their execution boundary.

⸻

16. Same-Sample Ordering

Commands targeting the same sample must have deterministic ordering.

Required rule:

lower sequence number executes first

Therefore:

command A
target = 48000
sequence = 10
command B
target = 48000
sequence = 11

must execute:

A
B

The sequence number must be generated by the producer.

⸻

17. Render-Quantum Boundary

The AudioWorklet render quantum is normally:

128 frames

The implementation must not assume that every invocation will always contain exactly 128 frames.

The kernel must continue to support arbitrary:

1..N

frame processing.

Commands must be executed at the exact sample boundary even when the boundary occurs inside a render quantum.

Example:

quantum:
48000 → 48127
command:
targetSample = 48064

The processor must split processing logically:

48000–48063
apply command
48064–48127

The resulting output must match offline execution.

⸻

18. Critical Determinism Test

This is the primary Phase 1 acceptance test.

Schedule:

targetSample = 48000
targetFreq = 32Hz

Render enough audio to cross the boundary.

Verify:

frame 47999 = old state
frame 48000 = new state

The browser result must agree with the Phase 0 offline result.

Acceptance tolerance must be explicitly documented.

⸻

19. Main-Thread Independence Test

The browser must prove that audio execution does not depend on the UI frame loop.

Test procedure:

Start audio.
Allow normal execution.
Block the main thread intentionally for approximately 500ms.
Resume main thread.
Inspect AudioWorklet telemetry.

Required result:

AudioWorklet sample counter remains monotonic.
No transport reset occurs.
No command reordering occurs.
No fatal AudioWorklet exception occurs.

The test must not claim zero audio dropouts unless the test environment can objectively measure that.

The minimum acceptance criterion is execution-plane independence.

⸻

20. Telemetry

Telemetry must travel from the execution plane to the main thread without requiring the audio engine to allocate objects on every render callback.

At minimum expose:

current_sample
active_voices
xruns
cpu_load_pct
peak_l
peak_r

The existing Phase 0 TelemetrySnapshot ABI remains:

32 bytes

Do not change the structure size in Phase 1 without a documented ABI revision.

⸻

21. Telemetry Transport

The AudioWorklet must write telemetry into preallocated shared memory or another explicitly bounded mechanism.

Do not perform:

JSON.stringify()
new Object()
new Array()
Vec
String allocation

inside the render callback merely to report telemetry.

Telemetry may be sampled by the main thread at a lower frequency.

The UI must never be required to acknowledge telemetry for audio to continue.

⸻

22. Render-Path Allocation Contract

The following are prohibited in the AudioWorklet render path:

new
Array
Object
Map
Set
JSON
Promise
async
fetch
console logging in the hot loop
filesystem operations
network operations
mutexes
blocking waits
WASM memory growth
dynamic DSP allocation

The following are allowed:

typed-array access
preallocated buffers
Atomics
bounded loops
WASM DSP calls
sample counter advancement
SPSC reads
SPSC writes

Any exception must be documented and tested.

⸻

23. Required Stress Test

Create a browser stress test with:

500 parameter/command updates per second
120Hz main-thread rendering workload
continuous AudioWorklet execution

The test must run for a defined duration.

Minimum:

30 seconds

Record:

commands submitted
commands consumed
commands executed
commands rejected
current sample
xruns
AudioWorklet errors

Acceptance:

No command corruption.
No queue framing corruption.
No fatal AudioWorklet exception.
Sample counter remains monotonic.

If the environment produces an actual xrun, record it rather than hiding it.

⸻

24. Phase 1 File Structure

The implementation may add files, but the expected minimum organization is:

browser-daw/
├── Cargo.toml
│
├── crates/
│   └── audio-kernel/
│       └── src/
│           ├── lib.rs
│           ├── abi.rs
│           ├── kernel.rs
│           ├── offline.rs
│           └── spsc.rs
│
├── web/
│   └── audio/
│       ├── wasm-loader.ts
│       ├── audio-kernel-processor.ts
│       ├── audio-kernel-node.ts
│       ├── sab-transport.ts
│       ├── telemetry.ts
│       └── protocol.ts
│
├── tests/
│   ├── wasm/
│   ├── browser/
│   └── integration/
│
└── LOG.md

Do not create the React application yet unless required purely as a browser test harness.

A minimal HTML/TypeScript test harness is preferred.

⸻

25. Build Pipeline

The browser build must produce:

audio_kernel.wasm

and make that artifact available to the browser test harness.

The exact Rust-to-WASM export mechanism may be:

#[export_name]

or another explicit ABI mechanism.

Do not introduce wasm-bindgen solely because it is convenient if it adds unnecessary runtime or generated binding complexity.

The selected mechanism must be documented.

⸻

26. Phase 1 Validation Commands

After implementation:

cargo fmt --all -- --check

Expected:

EXIT 0

Then:

cargo clippy --workspace --all-targets --all-features -- -D warnings

Expected:

EXIT 0

Then:

cargo test --workspace

Expected:

ALL TESTS PASS

Then:

cargo build --workspace --target wasm32-unknown-unknown

Expected:

EXIT 0

Then run the JavaScript/TypeScript test suite.

Expected:

ALL TESTS PASS

Then run browser integration tests.

Expected:

ALL REQUIRED PHASE 1 TESTS PASS

⸻

27. Phase 1 Acceptance Matrix

Requirement	Acceptance
Phase 0 tests	PASS
Rust formatting	PASS
Clippy -D warnings	PASS
Native build	PASS
WASM build	PASS
Explicit WASM exports	PASS
WASM loader	PASS
AudioWorklet initialization	PASS
WASM executes inside Worklet	PASS
Browser produces audio	PASS
SAB available	PASS
crossOriginIsolated	TRUE
SPSC command transport	PASS
Complete record boundaries	PASS
Sample-stamped commands	PASS
Future commands remain pending	PASS
Same-sample ordering deterministic	PASS
Command executes exact target sample	PASS
Quantum-boundary determinism	PASS
Telemetry transport	PASS
Main-thread independence	PASS
500 commands/sec stress test	PASS
No render-path allocation	PASS/AUDITED
No Phase 2 features	TRUE

⸻

28. Failure Classification

When a test fails, classify it before modifying code.

F1 — Phase 0 Regression

Any existing Phase 0 test fails.

Action:

STOP PHASE 1.
Restore Phase 0 invariant.

⸻

F2 — ABI Failure

WASM exports are missing, incorrectly typed, or unsafe.

Action:

Do not modify AudioWorklet.
Fix ABI.

⸻

F3 — Browser Environment Failure

Examples:

crossOriginIsolated === false
SAB unavailable
WASM unavailable
AudioWorklet unavailable

Action:

Report environment failure.
Do not implement a silent fallback.

⸻

F4 — Synchronization Failure

Examples:

command reordered
partial record consumed
incorrect sample boundary
telemetry races

Action:

Fix synchronization layer.
Do not compensate in React/UI code.

⸻

F5 — Kernel Determinism Failure

Browser result differs from offline result.

Action:

Fix execution boundary or kernel contract.
Do not loosen the test tolerance without evidence.

⸻

F6 — Real-Time Contract Failure

Examples:

allocation
blocking
unbounded queue
WASM memory growth
filesystem access
network access

Action:

STOP.
Remove violation before proceeding.

⸻

F7 — Scope Creep

Implementation begins adding:

React
OPFS
MIDI
effects
mixer
Gemini
timeline

Action:

STOP.
Remove Phase 2+ work.

⸻

29. Phase 1 Completion Criteria

Phase 1 is complete only when:

Phase 0 remains green
        AND
WASM ABI is explicit
        AND
WASM loads in browser
        AND
AudioWorklet executes kernel
        AND
SAB transport functions
        AND
commands are sample-stamped
        AND
commands execute deterministically
        AND
offline/browser behavior agrees
        AND
main-thread starvation does not corrupt execution
        AND
stress test passes
        AND
real-time allocation contract is satisfied

No UI quality criterion exists in Phase 1.

No DAW usability criterion exists in Phase 1.

The only question is:

Can the deterministic Phase 0 kernel execute reliably inside the browser’s real-time execution environment through an explicit WASM/SAB/AudioWorklet boundary?

If yes, Phase 1 passes.

⸻

30. Phase 1 Deliverable

At completion, Kilo must append a new entry to LOG.md containing:

Date
Time
Log Entry No.
Header/Title
Implementation Summary
Files Added
Files Modified
Commands Executed
Test Results
Browser Test Results
Stress Test Results
ABI Changes
Known Limitations
Phase 1 Acceptance Matrix
Phase 2 Readiness Status

The log must report observed results, not expected results.

No “PASS” may be recorded unless the corresponding test actually executed successfully.

⸻

31. Explicit Phase 2 Boundary

Phase 1 completion does NOT authorize:

OPFS
audio asset streaming
Canvas
WebGL
WebGPU
React UI
Zustand
mixer
multi-voice
effects
Gemini

Those remain Phase 2+ work.

The repository should therefore finish Phase 1 with a technically ugly but verifiably correct browser audio execution harness.

That is intentional.

Correctness precedes presentation.




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
