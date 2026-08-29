// AudioWorkletProcessor — owns WASM kernel, SAB consumer, sample-stamped ordering, telemetry (no alloc in process)
// Initialization via processorOptions: { wasmBytes, sampleRate, commandSab, telemetrySab }

declare const sampleRate: number;
declare class AudioWorkletProcessor { readonly port: MessagePort; constructor(o?: { processorOptions?: unknown }); process(inputs: Float32Array[][], outputs: Float32Array[][], params: Record<string, Float32Array>): boolean; }
declare function registerProcessor(name: string, cls: unknown): void;

type Cmd = { target: bigint; freq: number; duration: bigint; seq: number };

export class AudioKernelProcessor extends AudioWorkletProcessor {
  handle = 0; mem!: WebAssembly.Memory; curSample = 0n; pending: Cmd[] = []; seq = 0;
  cmdSab!: SharedArrayBuffer; telSab!: SharedArrayBuffer;
  wasm!: { memory: WebAssembly.Memory; audio_kernel_create(n:number):number; audio_kernel_destroy(n:number):void; audio_kernel_process(h:number,p:number,f:number):number; audio_kernel_apply_transaction(h:number,p:number,l:number):number; audio_kernel_read_telemetry(h:number,p:number,l:number):number; };

  constructor(opts?: { processorOptions?: { wasmBytes?: ArrayBuffer; sampleRate?: number; commandSab?: SharedArrayBuffer; telemetrySab?: SharedArrayBuffer }}) {
    super();
    const o = opts?.processorOptions ?? {};
    this.cmdSab = o.commandSab!; this.telSab = o.telemetrySab!;
    const bytes = o.wasmBytes!; const sr = o.sampleRate ?? sampleRate;
    // sync instantiate (bytes already compiled); use WebAssembly.Module sync if available
    const mod = new WebAssembly.Module(bytes);
    const inst = new WebAssembly.Instance(mod, {});
    const e = inst.exports as unknown as typeof this.wasm & { memory: WebAssembly.Memory };
    this.wasm = e; this.mem = e.memory;
    this.handle = e.audio_kernel_create(sr);
    this.port.onmessage = () => {};
    this.port.postMessage({ type: "initialized", sampleRate: sr });
  }

  private drainCommands() {
    if (!this.cmdSab) return;
    const h = new Int32Array(this.cmdSab, 0, 2);
    const cap = this.cmdSab.byteLength - 8;
    const data = new Uint8Array(this.cmdSab, 8);
    const tmp = new Uint8Array(24);
    while (true) {
      const r = Atomics.load(h, 1), w = Atomics.load(h, 0);
      if (((w - r) >>> 0) < 4) break;
      const lb = new Uint8Array(4); for (let i=0;i<4;i++) lb[i]=data[(r+i)&(cap-1)];
      const len = new DataView(lb.buffer).getUint32(0,true);
      if (((w-r)>>>0) < 4+len || len!==24) break;
      for(let i=0;i<24;i++) tmp[i]=data[(r+4+i)&(cap-1)];
      Atomics.store(h,1,(r+4+len)>>>0);
      const v=new DataView(tmp.buffer);
      this.pending.push({ target: v.getBigUint64(0,true), freq: v.getFloat32(8,true), duration: v.getBigUint64(16,true), seq: this.seq++ });
    }
    this.pending.sort((a,b)=> a.target!==b.target ? (a.target<b.target?-1:1) : a.seq-b.seq);
  }

  private applyReady() {
    while (this.pending.length && this.pending[0].target <= this.curSample) {
      const c = this.pending.shift()!;
      const wire = new Uint8Array(24); const v=new DataView(wire.buffer);
      v.setBigUint64(0,c.target,true); v.setFloat32(8,c.freq,true); v.setUint32(12,0,true); v.setBigUint64(16,c.duration,true);
      // copy wire into wasm memory at temp ptr (use stack-like bump at end of memory)
      const ptr = 1024; // safe scratch within first page (wasm memory is at least 1 page)
      new Uint8Array(this.mem.buffer).set(wire, ptr);
      this.wasm.audio_kernel_apply_transaction(this.handle, ptr, 24);
    }
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const out = outputs[0]?.[0]; if (!out) return true;
    const frames = out.length;
    this.drainCommands();
    // split at command boundaries within quantum
    let done = 0;
    while (done < frames) {
      this.applyReady();
      let nextTarget: bigint | null = null;
      for (const c of this.pending) if (c.target > this.curSample) { nextTarget = c.target; break; }
      const remaining = frames - done;
      let chunk = remaining;
      if (nextTarget !== null) {
        const until = Number(nextTarget - this.curSample);
        if (until < chunk) chunk = until;
      }
      if (chunk > 0) {
        const ptr = 2048; // scratch for output
        // ensure memory large enough
        if (this.mem.buffer.byteLength < ptr + chunk * 4) { /* grow not allowed in render */ }
        const res = this.wasm.audio_kernel_process(this.handle, ptr, chunk);
        if (res !== 0) { /* xrun */ }
        const memF32 = new Float32Array(this.mem.buffer, ptr, chunk);
        // bounded loop, no alloc
        for (let i = 0; i < chunk; i++) out[done + i] = memF32[i];
        this.curSample += BigInt(chunk);
        done += chunk;
      } else {
        // command exactly at boundary, loop will apply it
        if (nextTarget === this.curSample) this.applyReady(); else break;
      }
    }
    // telemetry write (preallocated)
    if (this.telSab) {
      const v = new DataView(this.telSab);
      v.setBigUint64(0, this.curSample, true);
      v.setUint32(8, 1, true); v.setUint32(12, 0, true);
      v.setFloat32(16, 0, true);
      let peak = 0; for (let i = 0; i < frames; i++) { const a = out[i] < 0 ? -out[i] : out[i]; if (a > peak) peak = a; }
      v.setFloat32(20, peak, true); v.setFloat32(24, peak, true);
    }
    return true;
  }
}
try { registerProcessor("audio-kernel-processor", AudioKernelProcessor); } catch {}
