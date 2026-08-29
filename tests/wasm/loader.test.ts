import { describe, it, expect } from "vitest";
import { readFile } from "node:fs/promises";
import { loadWasm } from "../../web/audio/wasm-loader";

describe("wasm loader", () => {
  it("resolves required exports and creates handle", async () => {
    const bytes = await readFile("target/wasm32-unknown-unknown/debug/audio_kernel.wasm");
    const mod = await loadWasm(bytes);
    expect(mod.memory).toBeDefined();
    const h = mod.audio_kernel_create(48000);
    expect(h).not.toBe(0);
    mod.audio_kernel_destroy(h);
  });
  it("rejects missing exports", async () => {
    const empty = await WebAssembly.compile(new Uint8Array([0x00,0x61,0x73,0x6d,0x01,0x00,0x00,0x00]));
    // instantiate empty module should throw missing export
    await expect(loadWasm(empty)).rejects.toThrow(/missing export/);
  });
});
