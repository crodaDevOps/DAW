// 24B LE wire: 0:u64 target_sample, 8:f32 target_freq, 12:u32 reserved, 16:u64 duration
export const WIRE_SIZE = 24;
export function encodeWire(targetSample: bigint, targetFreq: number, duration: bigint): Uint8Array {
  const b = new Uint8Array(24);
  const v = new DataView(b.buffer);
  v.setBigUint64(0, targetSample, true);
  v.setFloat32(8, targetFreq, true);
  v.setUint32(12, 0, true);
  v.setBigUint64(16, duration, true);
  return b;
}
