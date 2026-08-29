use crate::abi::{AudioKernel, MockTransaction, SampleFrame, TelemetrySnapshot};
pub struct DawKernel {
    sample_rate: f32,
    current_frame: SampleFrame,
    phase: f64,
    base_freq: f32,
    target_freq: f32,
    glide_start: SampleFrame,
    glide_duration: SampleFrame,
}
impl DawKernel {
    pub fn new(sample_rate: f32) -> Self {
        assert!(sample_rate > 0.0);
        Self {
            sample_rate,
            current_frame: 0,
            phase: 0.0,
            base_freq: 55.0,
            target_freq: 55.0,
            glide_start: u64::MAX,
            glide_duration: 0,
        }
    }
    #[inline]
    fn decode_transaction(payload: &[u8]) -> Option<MockTransaction> {
        if payload.len() != core::mem::size_of::<MockTransaction>() {
            return None;
        }
        let mut tx = MockTransaction {
            target_sample: 0,
            target_freq: 0.0,
            duration_samples: 0,
        };
        unsafe {
            core::ptr::copy_nonoverlapping(
                payload.as_ptr(),
                &mut tx as *mut MockTransaction as *mut u8,
                core::mem::size_of::<MockTransaction>(),
            );
        }
        Some(tx)
    }
}
impl AudioKernel for DawKernel {
    fn apply_transaction(&mut self, payload: &[u8]) {
        let Some(tx) = Self::decode_transaction(payload) else {
            return;
        };
        self.glide_start = tx.target_sample;
        self.target_freq = tx.target_freq;
        self.glide_duration = tx.duration_samples;
    }
    fn process(&mut self, _input: &[&[f32]], output: &mut [&mut [f32]], frames: usize) {
        assert!(output.iter().all(|channel| channel.len() >= frames));
        for i in 0..frames {
            let mut current_frequency = self.base_freq;
            if self.current_frame >= self.glide_start {
                if self.glide_duration == 0 {
                    current_frequency = self.target_freq;
                    self.base_freq = self.target_freq;
                    self.glide_start = u64::MAX;
                } else {
                    let elapsed = self.current_frame - self.glide_start;
                    if elapsed < self.glide_duration {
                        let t = elapsed as f32 / self.glide_duration as f32;
                        let ratio = self.target_freq / self.base_freq;
                        current_frequency = self.base_freq * libm::powf(ratio, t);
                    } else {
                        current_frequency = self.target_freq;
                        self.base_freq = self.target_freq;
                        self.glide_start = u64::MAX;
                        self.glide_duration = 0;
                    }
                }
            }
            let sample = libm::sinf((self.phase as f32) * core::f32::consts::TAU);
            for channel in output.iter_mut() {
                channel[i] = sample;
            }
            self.phase += f64::from(current_frequency) / f64::from(self.sample_rate);
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            self.current_frame += 1;
        }
    }
    fn read_telemetry(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            current_sample: self.current_frame,
            active_voices: 1,
            xruns: 0,
            cpu_load_pct: 0.0,
            peak_l: 1.0,
            peak_r: 1.0,
        }
    }
}
