import { createSAB } from "./sab-transport";
import { createTelemetrySAB, readTelemetry } from "./telemetry";

export async function createAudioKernelNode(ctx: AudioContext, wasmBytes: ArrayBuffer) {
  const commandSab = createSAB(4096);
  const telemetrySab = createTelemetrySAB();
  await ctx.audioWorklet.addModule(new URL("./audio-kernel-processor.ts", import.meta.url).toString());
  // fallback: if bundler, caller should ensure processor registered; we also support direct addModule via blob
  const node = new AudioWorkletNode(ctx, "audio-kernel-processor", {
    processorOptions: { wasmBytes, sampleRate: ctx.sampleRate, commandSab, telemetrySab }
  });
  await new Promise<void>(res => { node.port.onmessage = e => { if (e.data?.type === "initialized") res(); }; });
  return { node, commandSab, telemetrySab, readTelemetry: () => readTelemetry(telemetrySab) };
}
