import { type InvokeArgs, invoke } from "@tauri-apps/api/core";

const UTF8 = new TextDecoder();

type RawBytes = ArrayBuffer | ArrayBufferView | readonly number[];

export const unpacked = (bytes: RawBytes): string =>
  UTF8.decode(
    Array.isArray(bytes) ? Uint8Array.from(bytes) : (bytes as ArrayBuffer | ArrayBufferView),
  );

export const invokedText = (cmd: string, args: InvokeArgs): Promise<string> =>
  invoke<RawBytes>(cmd, args).then(unpacked);
