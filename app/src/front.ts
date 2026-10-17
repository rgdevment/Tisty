export interface Field {
  key: string;
  values: string[];
}

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

// The properties at the top of a note, read the way the core's `front::fields` reads them.
export const fields = (body: string): Field[] => {
  const lines = body.replace(/^﻿/, "").split("\n");
  if (lines[0]?.trimEnd() !== "---" || !lines[1]?.trim()) return [];
  const shut = lines.findIndex((line, at) => at > 1 && /^(---|\.\.\.)$/.test(line.trimEnd()));
  if (shut < 0) return [];
  const out: Field[] = [];
  let listing = false;
  for (const line of lines.slice(1, shut)) {
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
  return out;
};
