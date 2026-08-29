export interface AudioKernelWasm {
  memory: WebAssembly.Memory;
  audio_kernel_create(sample_rate: number): number;
  audio_kernel_destroy(handle: number): void;
  audio_kernel_process(handle: number, outPtr: number, frames: number): number;
  audio_kernel_apply_transaction(handle: number, ptr: number, len: number): number;
  audio_kernel_read_telemetry(handle: number, outPtr: number, len: number): number;
}

const REQUIRED = [
  "memory",
  "audio_kernel_create",
  "audio_kernel_destroy",
  "audio_kernel_process",
  "audio_kernel_apply_transaction",
  "audio_kernel_read_telemetry",
] as const;

export async function loadWasm(source: Response | BufferSource | WebAssembly.Module): Promise<AudioKernelWasm> {
  let instance: WebAssembly.Instance;
  if (source instanceof WebAssembly.Module) {
    instance = await WebAssembly.instantiate(source, {});
  } else if (source instanceof Response) {
    const buf = await source.arrayBuffer();
    const r = await WebAssembly.instantiate(buf, {});
    instance = r.instance;
  } else {
    const r = await WebAssembly.instantiate(source as BufferSource, {});
    instance = (r as { instance: WebAssembly.Instance }).instance ?? (r as unknown as WebAssembly.Instance);
  }
  const exports = instance.exports as Record<string, unknown>;
  for (const k of REQUIRED) if (!(k in exports)) throw new Error(`missing export: ${k}`);
  return {
    memory: exports.memory as WebAssembly.Memory,
    audio_kernel_create: exports.audio_kernel_create as AudioKernelWasm["audio_kernel_create"],
    audio_kernel_destroy: exports.audio_kernel_destroy as AudioKernelWasm["audio_kernel_destroy"],
    audio_kernel_process: exports.audio_kernel_process as AudioKernelWasm["audio_kernel_process"],
    audio_kernel_apply_transaction: exports.audio_kernel_apply_transaction as AudioKernelWasm["audio_kernel_apply_transaction"],
    audio_kernel_read_telemetry: exports.audio_kernel_read_telemetry as AudioKernelWasm["audio_kernel_read_telemetry"],
  };
}
