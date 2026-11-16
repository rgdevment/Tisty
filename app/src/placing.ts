import { invoke } from "@tauri-apps/api/core";

type Hash = readonly [number, number];

export type Check = { readonly bytes: number; readonly hash: Hash };

const SEED: Hash = [0x811c9dc5, 0x9747b28c];

const summedBy = (
  count: number,
  numberAt: (at: number) => number,
  seed = SEED,
): [number, number] => {
  let [fnv, mixed] = seed;
  for (let at = 0; at < count; at++) {
    const one = numberAt(at);
    fnv = Math.imul(fnv ^ one, 0x01000193);
    mixed = Math.imul(mixed ^ one, 0x5bd1e995);
    mixed ^= mixed >>> 13;
  }
  return [fnv >>> 0, mixed >>> 0];
};

export const summed = (numbers: readonly number[], seed = SEED) =>
  summedBy(numbers.length, (at) => numbers[at], seed);

const unitsSummed = (text: string, seed = SEED) =>
  summedBy(text.length, (at) => text.charCodeAt(at), seed);

const surrogate = (unit: number) => unit >= 0xd800 && unit < 0xe000;

export const bytesIn = (text: string, from = 0, to = text.length) => {
  let bytes = 0;
  for (let at = from; at < to; at++) {
    const unit = text.charCodeAt(at);
    bytes += unit < 0x80 ? 1 : unit < 0x800 || surrogate(unit) ? 2 : 3;
  }
  return bytes;
};

export const checkOf = (text: string): Check => ({ bytes: bytesIn(text), hash: unitsSummed(text) });

// A key's check follows from the one it was typed on, so naming the new text never reads it whole.
export const checkAfter = (was: Check, edit: Typed): Check => ({
  bytes: was.bytes - (edit.to - edit.from) + bytesIn(edit.insert),
  hash: unitsSummed(edit.insert, summed([edit.from, edit.to, edit.insert.length], was.hash)),
});

export const docPlaced = (body: string): Promise<string> =>
  invoke<ArrayBuffer>("doc_placed", { body, check: checkOf(body) }).then((bytes) =>
    new TextDecoder().decode(bytes),
  );

export type PlacedNear = {
  readonly count: number;
  readonly sum: readonly [number, number];
  readonly from: number;
  readonly to: number;
  readonly html: string;
};

export type Typed = { readonly from: number; readonly to: number; readonly insert: string };

export type Sent = {
  readonly body: string;
  readonly was: Check;
  readonly now: Check;
  readonly edit: Typed;
};

// Only the key crosses, and the whole body follows when the core holds another text than the one it was typed on.
export const docPlacedNear = async (
  { body, was, now, edit }: Sent,
  from: number,
  to: number,
): Promise<PlacedNear> =>
  (await invoke<PlacedNear | null>("doc_placed_near", {
    sent: { edit: { ...edit, was } },
    now,
    from,
    to,
  })) ?? invoke<PlacedNear>("doc_placed_near", { sent: { body }, now, from, to });
