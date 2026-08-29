// 32B TelemetrySnapshot: 0:u64 current_sample, 8:u32 active_voices,12:u32 xruns,16:f32 cpu,20:f32 peak_l,24:f32 peak_r
export function createTelemetrySAB(): SharedArrayBuffer { return new SharedArrayBuffer(32); }
export function writeTelemetry(sab: SharedArrayBuffer, s: {current:bigint, voices:number, xruns:number, cpu:number, peakL:number, peakR:number}) {
  const v = new DataView(sab); v.setBigUint64(0, s.current, true); v.setUint32(8, s.voices, true); v.setUint32(12, s.xruns, true); v.setFloat32(16, s.cpu, true); v.setFloat32(20, s.peakL, true); v.setFloat32(24, s.peakR, true);
}
export function readTelemetry(sab: SharedArrayBuffer) {
  const v = new DataView(sab); return { current_sample: v.getBigUint64(0,true), active_voices: v.getUint32(8,true), xruns: v.getUint32(12,true), cpu_load_pct: v.getFloat32(16,true), peak_l: v.getFloat32(20,true), peak_r: v.getFloat32(24,true)};
}
