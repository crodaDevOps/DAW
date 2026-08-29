use core::mem::size_of;
pub type SampleFrame = u64;
pub const AUDIO_RENDER_QUANTUM: usize = 128;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySnapshot {
    pub current_sample: SampleFrame,
    pub active_voices: u32,
    pub xruns: u32,
    pub cpu_load_pct: f32,
    pub peak_l: f32,
    pub peak_r: f32,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MockTransaction {
    pub target_sample: SampleFrame,
    pub target_freq: f32,
    pub duration_samples: SampleFrame,
}
pub const MOCK_TRANSACTION_SIZE: usize = size_of::<MockTransaction>();
const _: () = assert!(size_of::<MockTransaction>() == 24);
const _: () = assert!(size_of::<TelemetrySnapshot>() == 32);
/// The real-time execution contract.
///
/// Implementations must:
///
/// - not allocate
/// - not block
/// - not perform I/O
/// - not invoke OS syscalls
/// - not acquire mutexes
/// - operate deterministically for identical inputs/state
pub trait AudioKernel {
    /// Process exactly `frames` audio frames.
    ///
    /// `output[channel][frame]`
    fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]], frames: usize);
    /// Apply a complete transaction at a render boundary.
    fn apply_transaction(&mut self, payload: &[u8]);
    /// Return a point-in-time telemetry snapshot.
    fn read_telemetry(&self) -> TelemetrySnapshot;
}
