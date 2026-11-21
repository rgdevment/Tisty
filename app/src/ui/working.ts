import type { Which } from "./Card";

export type Working = ReadonlyMap<Which, number>;

export const IDLE: Working = new Map();

// Everything that reads or rewrites the history, the shared folder or the cloud waits for the others.
export const YOUR_DATA: readonly Which[] = [
  "sync",
  "syncOff",
  "cloudForget",
  "cloudRevoke",
  "cloudAbandon",
  "sealWipe",
  "sealOff",
  "backup",
  "restore",
  "sealing",
  "identity",
  "machines",
  "keys",
  "parcel",
  "review",
  "recover",
  "missing",
  "unread",
  "rebuild",
  "index",
  "loose",
  "twins",
  "free",
  "holds",
  "repeated",
  "unvouched",
  "store",
];

export const GROUPS: readonly (readonly Which[])[] = [YOUR_DATA];

const together = (one: Which, other: Which): boolean =>
  one === other || GROUPS.some((group) => group.includes(one) && group.includes(other));

export const holder = (busy: Working, which: Which): Which | null => {
  if (busy.has(which)) return which;
  for (const at of busy.keys()) if (together(at, which)) return at;
  return null;
};

export const held = (busy: Working, which: Which): boolean => holder(busy, which) !== null;
