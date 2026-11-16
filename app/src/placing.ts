import { invoke } from "@tauri-apps/api/core";

export type Check = { readonly bytes: number; readonly hash: number };

const surrogate = (unit: number) => unit >= 0xd800 && unit < 0xe000;

export const checkOf = (text: string): Check => {
  let utf8 = 0;
  let fnv = 0x811c9dc5;
  for (let at = 0; at < text.length; at++) {
    const unit = text.charCodeAt(at);
    utf8 += unit < 0x80 ? 1 : unit < 0x800 || surrogate(unit) ? 2 : 3;
    fnv = Math.imul(fnv ^ unit, 0x01000193);
  }
  return { bytes: utf8, hash: fnv >>> 0 };
};

export const docPlaced = (body: string): Promise<string> =>
  invoke<ArrayBuffer>("doc_placed", { body, check: checkOf(body) }).then((bytes) =>
    new TextDecoder().decode(bytes),
  );

export type PlacedNear = {
  readonly spans: readonly (readonly [number, number])[];
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
