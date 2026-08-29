// SAB layout: [0:u32 write_idx][4:u32 read_idx][8: byte storage]
// Compatible with kernel spsc framing: [u32 LE len][payload]
export const HEADER = 8;
export function createSAB(capacity: number): SharedArrayBuffer {
  if ((capacity & (capacity - 1)) !== 0) throw new Error("capacity pow2");
  return new SharedArrayBuffer(HEADER + capacity);
}
export function sabProducer(sab: SharedArrayBuffer) {
  const h = new Int32Array(sab, 0, 2);
  const d = new Uint8Array(sab, HEADER);
  const cap = d.length;
  return {
    push(payload: Uint8Array): boolean {
      const w = Atomics.load(h, 0), r = Atomics.load(h, 1);
      const used = (w - r) >>> 0, free = cap - used;
      const need = 4 + payload.length;
      if (need >= cap || free < need) return false;
      const len = new Uint8Array(4); new DataView(len.buffer).setUint32(0, payload.length, true);
      for (let i = 0; i < 4; i++) d[(w + i) & (cap - 1)] = len[i];
      for (let i = 0; i < payload.length; i++) d[(w + 4 + i) & (cap - 1)] = payload[i];
      Atomics.store(h, 0, (w + need) >>> 0);
      Atomics.notify(h, 0, 1);
      return true;
    }
  };
}
export function sabConsumer(sab: SharedArrayBuffer) {
  const h = new Int32Array(sab, 0, 2);
  const d = new Uint8Array(sab, HEADER);
  const cap = d.length;
  return {
    pop(out: Uint8Array): number | null {
      const r = Atomics.load(h, 1), w = Atomics.load(h, 0);
      if (((w - r) >>> 0) < 4) return null;
      const lb = new Uint8Array(4); for (let i = 0; i < 4; i++) lb[i] = d[(r + i) & (cap - 1)];
      const len = new DataView(lb.buffer).getUint32(0, true);
      if (((w - r) >>> 0) < 4 + len || len > out.length) return null;
      for (let i = 0; i < len; i++) out[i] = d[(r + 4 + i) & (cap - 1)];
      Atomics.store(h, 1, (r + 4 + len) >>> 0);
      return len;
    }
  };
}
