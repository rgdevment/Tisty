export interface Field {
  key: string;
  values: string[];
}

const BLOCK_AT_MOST = 4 * 1024;
const KEYS_AT_MOST = 32;
const VALUE_AT_MOST = 500;

const unquoted = (said: string): string => {
  const flat = said.trim();
  for (const quote of ['"', "'"])
    if (flat.length > 1 && flat.startsWith(quote) && flat.endsWith(quote)) return flat.slice(1, -1);
  return flat;
};

const listed = (value: string): string[] => {
  if (!value) return [];
  if (value.startsWith("[") && value.endsWith("]"))
    return value
      .slice(1, -1)
      .split(",")
      .map(unquoted)
      .filter((one) => one !== "");
  return [unquoted(value)];
};

const block = (body: string): string[] | null => {
  const lines = body.replace(/^﻿/, "").split("\n");
  if (lines[0]?.trimEnd() !== "---" || !lines[1]?.trim()) return null;
  const shut = lines.findIndex((line, at) => at > 1 && /^(---|\.\.\.)$/.test(line.trimEnd()));
  return shut < 0 ? null : lines.slice(0, shut + 1);
};

// Front matter past the caps is kept as written, but nothing in it is read.
export const overgrown = (body: string): boolean => {
  const held = block(body);
  return held !== null && new TextEncoder().encode(`${held.join("\n")}\n`).length > BLOCK_AT_MOST;
};

// The properties at the top of a note, read the way the core's `front::fields` reads them.
export const fields = (body: string): Field[] => {
  const lines = block(body);
  if (!lines || overgrown(body)) return [];
  const out: Field[] = [];
  let listing = false;
  for (const line of lines.slice(1, -1)) {
    const flat = line.trim();
    if (!flat || flat.startsWith("#")) continue;
    const last = out[out.length - 1];
    if (last && listing && flat.startsWith("- ")) {
      last.values.push(unquoted(flat.slice(2)));
      continue;
    }
    if (last && /^[ \t]/.test(line)) {
      last.values.push(flat);
      continue;
    }
    const colon = flat.indexOf(":");
    if (colon < 0) continue;
    const values = listed(flat.slice(colon + 1).trim());
    listing = values.length === 0;
    out.push({ key: flat.slice(0, colon).trim(), values });
  }
  const past =
    out.length > KEYS_AT_MOST ||
    out.some((one) => one.values.some((value) => [...value].length > VALUE_AT_MOST));
  return past ? [] : out;
};
