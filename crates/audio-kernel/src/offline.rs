#[cfg(test)]
mod tests {
    use crate::{
        abi::{AudioKernel, MockTransaction, AUDIO_RENDER_QUANTUM},
        kernel::DawKernel,
    };
    use std::vec;
    use std::vec::Vec;
    struct OfflineDriver {
        kernel: DawKernel,
    }
    impl OfflineDriver {
        fn new(sample_rate: f32) -> Self {
            Self {
                kernel: DawKernel::new(sample_rate),
            }
        }
        fn render(&mut self, total_frames: usize) -> Vec<f32> {
            let mut output = vec![0.0f32; total_frames];
            let mut processed = 0usize;
            while processed < total_frames {
                let frames = core::cmp::min(AUDIO_RENDER_QUANTUM, total_frames - processed);
                let chunk = &mut output[processed..processed + frames];
                let mut channels: [&mut [f32]; 1] = [chunk];
                self.kernel.process(&[], &mut channels, frames);
                processed += frames;
            }
            output
        }
        fn dispatch_transaction(&mut self, tx: MockTransaction) {
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    &tx as *const MockTransaction as *const u8,
                    core::mem::size_of::<MockTransaction>(),
                )
            };
            self.kernel.apply_transaction(bytes);
        }
    }
    #[test]
    fn deterministic_808_glide() {
        let sample_rate = 48_000.0;
        let mut driver = OfflineDriver::new(sample_rate);
        let transaction = MockTransaction {
            target_sample: 24_000,
            target_freq: 32.0,
            duration_samples: 4_800,
        };
        driver.dispatch_transaction(transaction);
        let rendered = driver.render(96_000);
        // Signal exists before transaction.
        assert!(rendered[23_999].abs() > 0.0);
        // Exact transaction boundary:
        //
        // 24000 * 55 / 48000 = 27.5 cycles
        //
        // fractional phase = 0.5
        // sin(PI) ≈ 0
        assert!(
            rendered[24_000].abs() < 0.0001,
            "transaction boundary phase mismatch: {}",
            rendered[24_000]
        );
        // After the glide has completed, measure the stabilized
        // oscillator frequency.
        let mut crossings = Vec::new();
        for i in 89_000..90_999 {
            if rendered[i] < 0.0 && rendered[i + 1] >= 0.0 {
                crossings.push(i);
            }
        }
        assert!(
            crossings.len() >= 2,
            "insufficient zero crossings: {}",
            crossings.len()
        );
        let period = crossings[1] - crossings[0];
        let measured_frequency = sample_rate / period as f32;
        assert!(
            (measured_frequency - 32.0).abs() < 0.5,
            "expected approximately 32Hz, measured {}Hz",
            measured_frequency
        );
        assert_eq!(driver.kernel.read_telemetry().current_sample, 96_000);
    }
    #[test]
    fn render_is_independent_of_quantum_partitioning() {
        let sample_rate = 48_000.0;
        let mut a = DawKernel::new(sample_rate);
        let mut b = DawKernel::new(sample_rate);
        let tx = MockTransaction {
            target_sample: 24_000,
            target_freq: 32.0,
            duration_samples: 4_800,
        };
        let bytes = unsafe {
            core::slice::from_raw_parts(
                &tx as *const MockTransaction as *const u8,
                core::mem::size_of::<MockTransaction>(),
            )
        };
        a.apply_transaction(bytes);
        b.apply_transaction(bytes);
        let mut output_a = vec![0.0f32; 4096];
        let mut output_b = vec![0.0f32; 4096];
        let mut position = 0;
        while position < 4096 {
            let frames = core::cmp::min(128, 4096 - position);
            let chunk = &mut output_a[position..position + frames];
            let mut channels = [chunk];
            a.process(&[], &mut channels, frames);
            position += frames;
        }
        position = 0;
        while position < 4096 {
            let frames = core::cmp::min(64, 4096 - position);
            let chunk = &mut output_b[position..position + frames];
            let mut channels = [chunk];
            b.process(&[], &mut channels, frames);
            position += frames;
        }
        assert_eq!(output_a, output_b);
    }
}
