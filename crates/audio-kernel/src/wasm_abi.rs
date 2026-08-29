use crate::abi::{AudioKernel as _, MockTransaction, TelemetrySnapshot, MOCK_TRANSACTION_SIZE};
use crate::kernel::DawKernel;
use core::mem::size_of;

// Wire format (24B LE): offset 0: u64 target_sample, 8: f32 target_freq, 12: u32 reserved, 16: u64 duration
const WIRE_SIZE: usize = 24;

fn decode_wire(bytes: &[u8]) -> Option<MockTransaction> {
    if bytes.len() != WIRE_SIZE {
        return None;
    }
    let target_sample = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    let target_freq = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
    // bytes 12..16 reserved, ignore
    let duration_samples = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    if !target_freq.is_finite() {
        return None;
    }
    Some(MockTransaction {
        target_sample,
        target_freq,
        duration_samples,
    })
}

/// Create kernel, return opaque pointer as usize (0 = failure)
#[no_mangle]
pub extern "C" fn audio_kernel_create(sample_rate: f32) -> usize {
    if !sample_rate.is_finite() || sample_rate <= 0.0 || sample_rate > 192000.0 {
        return 0;
    }
    let k = DawKernel::new(sample_rate);
    #[cfg(target_arch = "wasm32")]
    {
        let b = alloc::boxed::Box::new(k);
        return alloc::boxed::Box::into_raw(b) as usize;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let b = std::boxed::Box::new(k);
        #[allow(clippy::needless_return)]
        return std::boxed::Box::into_raw(b) as usize;
    }
}

#[no_mangle]
pub extern "C" fn audio_kernel_destroy(handle: usize) {
    if handle == 0 {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    unsafe {
        drop(alloc::boxed::Box::from_raw(handle as *mut DawKernel))
    };
    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        drop(std::boxed::Box::from_raw(handle as *mut DawKernel))
    };
}

/// Process: writes `frames` f32 samples to output_ptr (mono, non-interleaved)
/// Returns 0 ok, -1 invalid handle/ptr, -2 invalid frames
#[no_mangle]
pub extern "C" fn audio_kernel_process(handle: usize, output_ptr: usize, frames: usize) -> i32 {
    if handle == 0 || output_ptr == 0 || frames == 0 || frames > 8192 {
        return -1;
    }
    let kernel = unsafe { &mut *(handle as *mut DawKernel) };
    let out = unsafe { core::slice::from_raw_parts_mut(output_ptr as *mut f32, frames) };
    let mut channels: [&mut [f32]; 1] = [out];
    kernel.process(&[], &mut channels, frames);
    0
}

/// Apply transaction wire bytes. Returns 0 ok, -1 invalid handle/ptr, -2 bad length/encoding
#[no_mangle]
pub extern "C" fn audio_kernel_apply_transaction(
    handle: usize,
    data_ptr: usize,
    data_len: usize,
) -> i32 {
    if handle == 0 {
        return -1;
    }
    if data_ptr == 0 || data_len != WIRE_SIZE {
        return -2;
    }
    if data_len != MOCK_TRANSACTION_SIZE {
        return -2;
    }
    let bytes = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, data_len) };
    let Some(tx) = decode_wire(bytes) else {
        return -2;
    };
    let kernel = unsafe { &mut *(handle as *mut DawKernel) };
    // Re-encode as MockTransaction bytes via copy (kernel expects that layout)
    let tx_bytes = unsafe {
        core::slice::from_raw_parts(
            &tx as *const MockTransaction as *const u8,
            size_of::<MockTransaction>(),
        )
    };
    kernel.apply_transaction(tx_bytes);
    0
}

/// Read telemetry into out_ptr (must be >=32 bytes). Returns 0 ok, -1 invalid handle/ptr, -2 len
#[no_mangle]
pub extern "C" fn audio_kernel_read_telemetry(
    handle: usize,
    out_ptr: usize,
    out_len: usize,
) -> i32 {
    if handle == 0 || out_ptr == 0 {
        return -1;
    }
    if out_len < size_of::<TelemetrySnapshot>() {
        return -2;
    }
    let kernel = unsafe { &*(handle as *const DawKernel) };
    let snap = kernel.read_telemetry();
    unsafe {
        core::ptr::copy_nonoverlapping(
            &snap as *const TelemetrySnapshot as *const u8,
            out_ptr as *mut u8,
            size_of::<TelemetrySnapshot>(),
        );
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_callable() {
        let h = audio_kernel_create(48000.0);
        assert_ne!(h, 0);
        let mut out = [0f32; 128];
        assert_eq!(audio_kernel_process(h, out.as_mut_ptr() as usize, 128), 0);
        assert!(out.iter().any(|v| *v != 0.0));
        let mut tel = [0u8; 32];
        assert_eq!(
            audio_kernel_read_telemetry(h, tel.as_mut_ptr() as usize, tel.len()),
            0
        );
        audio_kernel_destroy(h);
    }
    #[test]
    fn invalid_rejected_safely() {
        assert_eq!(audio_kernel_process(0, 0, 0), -1);
        assert_eq!(audio_kernel_apply_transaction(0, 0, 0), -1);
        assert_eq!(audio_kernel_read_telemetry(0, 0, 0), -1);
        let h = audio_kernel_create(48000.0);
        assert_eq!(audio_kernel_apply_transaction(h, 0, 0), -2);
        assert_eq!(audio_kernel_apply_transaction(h, 0x1, 1), -2);
        let mut tel = [0u8; 4];
        assert_eq!(
            audio_kernel_read_telemetry(h, tel.as_mut_ptr() as usize, tel.len()),
            -2
        );
        // bad wire len
        let bad = [0u8; 10];
        assert_eq!(
            audio_kernel_apply_transaction(h, bad.as_ptr() as usize, bad.len()),
            -2
        );
        audio_kernel_destroy(h);
        // destroy 0 is noop
        audio_kernel_destroy(0);
    }
    #[test]
    fn valid_transaction_accepted() {
        let h = audio_kernel_create(48000.0);
        let mut wire = [0u8; 24];
        wire[0..8].copy_from_slice(&48000u64.to_le_bytes());
        wire[8..12].copy_from_slice(&32f32.to_le_bytes());
        wire[16..24].copy_from_slice(&4800u64.to_le_bytes());
        assert_eq!(
            audio_kernel_apply_transaction(h, wire.as_ptr() as usize, wire.len()),
            0
        );
        let mut out = [0f32; 96_000];
        // process via multiple calls to cross boundary
        let mut off = 0;
        while off < 96_000 {
            let n = core::cmp::min(128, 96_000 - off);
            assert_eq!(
                audio_kernel_process(h, unsafe { out.as_mut_ptr().add(off) } as usize, n),
                0
            );
            off += n;
        }
        assert!(out[24_000].abs() < 0.001);
        audio_kernel_destroy(h);
    }
    #[test]
    fn telemetry_readable() {
        let h = audio_kernel_create(48000.0);
        let mut buf = [0u8; 32];
        assert_eq!(
            audio_kernel_read_telemetry(h, buf.as_mut_ptr() as usize, 32),
            0
        );
        let mut snap = core::mem::MaybeUninit::<TelemetrySnapshot>::uninit();
        unsafe { core::ptr::copy_nonoverlapping(buf.as_ptr(), snap.as_mut_ptr() as *mut u8, 32) };
        let snap = unsafe { snap.assume_init() };
        assert_eq!(snap.current_sample, 0);
        audio_kernel_destroy(h);
    }
}
