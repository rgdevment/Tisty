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

export const namesSum = (names: readonly string[]) =>
  names.reduce<Hash>((sum, name) => unitsSummed(name, summed([name.length], sum)), SEED);

const surrogate = (unit: number) => unit >= 0xd800 && unit < 0xe000;

const widthOf = (unit: number) => {
  if (unit < 0x80) return 1;
  return unit < 0x800 || surrogate(unit) ? 2 : 3;
};

export const bytesIn = (text: string, from = 0, to = text.length) => {
  let bytes = 0;
  for (let at = from; at < to; at++) bytes += widthOf(text.charCodeAt(at));
  return bytes;
};

export const checkOf = (text: string, bytes = bytesIn(text)): Check => ({
  bytes,
  hash: unitsSummed(text),
});

export type Reckoned = {
  readonly text: string;
  readonly check: Check;
  readonly unit: number;
  readonly byte: number;
};

export type Placed = Pick<Reckoned, "text" | "unit" | "byte">;

// A place is counted from where the last key landed while the text is that one, never from its start.
export const byteFrom = (known: Placed | null, text: string, unit: number) => {
  if (known?.text !== text) return bytesIn(text, 0, unit);
  return unit >= known.unit
    ? known.byte + bytesIn(text, known.unit, unit)
    : known.byte - bytesIn(text, unit, known.unit);
};

export const unitFrom = (known: Reckoned | null, text: string, byte: number) => {
  const mine = known?.text === text;
  let unit = mine ? known.unit : 0;
  let at = mine ? known.byte : 0;
  while (at < byte && unit < text.length) at += widthOf(text.charCodeAt(unit++));
  while (at > byte && unit > 0) at -= widthOf(text.charCodeAt(--unit));
  return unit;
};

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
  readonly heads: readonly [number, number] | null;
  readonly whole: boolean;
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
