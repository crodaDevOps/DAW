use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};
#[repr(C, align(64))]
struct CacheAligned<T>(T);
#[repr(C)]
pub struct ByteRingBuffer<const CAP: usize> {
    write_idx: CacheAligned<AtomicUsize>,
    read_idx: CacheAligned<AtomicUsize>,
    data: UnsafeCell<[u8; CAP]>,
}
unsafe impl<const CAP: usize> Sync for ByteRingBuffer<CAP> {}
impl<const CAP: usize> ByteRingBuffer<CAP> {
    pub const fn new() -> Self {
        assert!(CAP.is_power_of_two());
        assert!(CAP >= 8);
        Self {
            write_idx: CacheAligned(AtomicUsize::new(0)),
            read_idx: CacheAligned(AtomicUsize::new(0)),
            data: UnsafeCell::new([0; CAP]),
        }
    }
    #[inline]
    fn used(write: usize, read: usize) -> usize {
        write.wrapping_sub(read)
    }
    #[inline]
    fn free(write: usize, read: usize) -> usize {
        CAP - Self::used(write, read)
    }
    #[inline]
    fn write_bytes(&self, start: usize, bytes: &[u8]) {
        let mask = CAP - 1;
        let data = unsafe { &mut *self.data.get() };
        for (i, byte) in bytes.iter().enumerate() {
            data[(start + i) & mask] = *byte;
        }
    }
    #[inline]
    fn read_bytes(&self, start: usize, out: &mut [u8]) {
        let mask = CAP - 1;
        let data = unsafe { &*self.data.get() };
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = data[(start + i) & mask];
        }
    }
    /// Push an entire record.
    ///
    /// Record format:
    ///
    /// [u32 little-endian payload length][payload bytes]
    ///
    /// Returns false if the entire record cannot fit.
    pub fn push_record(&self, payload: &[u8]) -> bool {
        let record_size = 4usize.saturating_add(payload.len());
        if record_size >= CAP {
            return false;
        }
        let write = self.write_idx.0.load(Ordering::Relaxed);
        let read = self.read_idx.0.load(Ordering::Acquire);
        if Self::free(write, read) < record_size {
            return false;
        }
        let len = payload.len() as u32;
        self.write_bytes(write, &len.to_le_bytes());
        self.write_bytes(write.wrapping_add(4), payload);
        self.write_idx
            .0
            .store(write.wrapping_add(record_size), Ordering::Release);
        true
    }
    /// Pop one complete record.
    ///
    /// Returns:
    ///
    /// Some(length) when a complete record was read.
    /// None when no complete record is currently available.
    pub fn pop_record(&self, out: &mut [u8]) -> Option<usize> {
        let read = self.read_idx.0.load(Ordering::Relaxed);
        let write = self.write_idx.0.load(Ordering::Acquire);
        if Self::used(write, read) < 4 {
            return None;
        }
        let mut length_bytes = [0u8; 4];
        self.read_bytes(read, &mut length_bytes);
        let length = u32::from_le_bytes(length_bytes) as usize;
        let record_size = 4usize.checked_add(length)?;
        if Self::used(write, read) < record_size {
            return None;
        }
        if length > out.len() {
            return None;
        }
        self.read_bytes(read.wrapping_add(4), &mut out[..length]);
        self.read_idx
            .0
            .store(read.wrapping_add(record_size), Ordering::Release);
        Some(length)
    }
    pub fn available(&self) -> usize {
        let write = self.write_idx.0.load(Ordering::Acquire);
        let read = self.read_idx.0.load(Ordering::Acquire);
        Self::used(write, read)
    }
}
impl<const CAP: usize> Default for ByteRingBuffer<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ByteRingBuffer;
    #[test]
    fn record_round_trip() {
        let ring = ByteRingBuffer::<64>::new();
        assert!(ring.push_record(b"hello"));
        let mut output = [0u8; 16];
        assert_eq!(ring.pop_record(&mut output), Some(5));
        assert_eq!(&output[..5], b"hello");
    }
    #[test]
    fn records_preserve_boundaries() {
        let ring = ByteRingBuffer::<64>::new();
        assert!(ring.push_record(b"one"));
        assert!(ring.push_record(b"two"));
        assert!(ring.push_record(b"three"));
        let mut output = [0u8; 16];
        assert_eq!(ring.pop_record(&mut output), Some(3));
        assert_eq!(&output[..3], b"one");
        assert_eq!(ring.pop_record(&mut output), Some(3));
        assert_eq!(&output[..3], b"two");
        assert_eq!(ring.pop_record(&mut output), Some(5));
        assert_eq!(&output[..5], b"three");
        assert_eq!(ring.pop_record(&mut output), None);
    }
    #[test]
    fn oversized_record_is_rejected() {
        let ring = ByteRingBuffer::<64>::new();
        let payload = [0u8; 64];
        assert!(!ring.push_record(&payload));
    }
    #[test]
    fn insufficient_output_buffer_does_not_consume_record() {
        let ring = ByteRingBuffer::<64>::new();
        assert!(ring.push_record(b"abcdef"));
        let mut small = [0u8; 3];
        assert_eq!(ring.pop_record(&mut small), None);
        let mut large = [0u8; 8];
        assert_eq!(ring.pop_record(&mut large), Some(6));
        assert_eq!(&large[..6], b"abcdef");
    }
}
