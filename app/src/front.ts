import type { Paper } from "./core";
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

export const frontLines = (body: string): string[] | null => {
  const lines = body.replace(/^﻿/, "").split("\n");
  if (lines[0]?.trimEnd() !== "---" || !lines[1]?.trim()) return null;
  const shut = lines.findIndex((line, at) => at > 1 && /^(---|\.\.\.)$/.test(line.trimEnd()));
  return shut < 0 ? null : lines.slice(0, shut + 1);
};

export const unfronted = (body: string): string => {
  const held = frontLines(body);
  if (!held) return body;
  return body.replace(/^﻿/, "").split("\n").slice(held.length).join("\n");
};

// Front matter past the caps is kept as written, but nothing in it is read.
export const overgrown = (body: string): boolean => {
  const held = frontLines(body);
  return held !== null && new TextEncoder().encode(`${held.join("\n")}\n`).length > BLOCK_AT_MOST;
};

export const fields = (body: string): Field[] => {
  const lines = frontLines(body);
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

export const frontTags = (body: string): string[] =>
  fields(body)
    .filter((one) => /^tags?$/i.test(one.key))
    .flatMap((one) => one.values.flatMap((value) => value.split(/[, ]/)))
    .map((word) => word.replace(/^#+/, ""))
    .filter((word) => word !== "");

const PAPERS: Paper[] = ["a4", "letter", "tabloid"];

export type Laid = {
  readonly paper: Paper | null;
  readonly sheets: boolean;
  readonly landscape: boolean;
};

export const laidIn = (body: string): Laid => {
  const all = fields(body);
  const said = (key: string) =>
    all.find((one) => one.key.toLowerCase() === key)?.values[0]?.toLowerCase();
  const sheets = said("sheets") === "on";
  return {
    paper: PAPERS.find((one) => one === said("paper")) ?? null,
    sheets,
    landscape: sheets && said("orientation") === "landscape",
  };
};

export const paperIn = (body: string): Paper | null => laidIn(body).paper;

export const landscapeIn = (body: string): boolean => laidIn(body).landscape;

export const sheetsIn = (body: string): boolean => laidIn(body).sheets;

// What a page is unless it says otherwise is never written down, so only a change from it is.
const withProperty = (body: string, key: string, value: string | null): string => {
  const line = body.includes("\r\n") ? "\r\n" : "\n";
  const lead = body.startsWith("\uFEFF") ? "\uFEFF" : "";
  const plain = body.slice(lead.length).split(line).join("\n");
  const held = frontLines(plain);
  const added = value === null ? [] : [`${key}: ${value}`];
  let made: string;
  if (!held) {
    made = added.length ? `---\n${added[0]}\n---\n${plain}` : plain;
  } else {
    const named = new RegExp(`^${key}\\s*:`, "i");
    const inside = held.slice(1, -1).filter((one) => !named.test(one.trim()));
    const kept = [...inside, ...added];
    const rest = plain.split("\n").slice(held.length).join("\n");
    made = kept.some((one) => one.trim())
      ? `---\n${kept.join("\n")}\n${held[held.length - 1]}\n${rest}`
      : rest.replace(/^\n/, "");
  }
  return lead + made.split("\n").join(line);
};

export const withPaper = (body: string, paper: Paper): string =>
  withProperty(body, "paper", paper === "a4" ? null : paper);

export const withLandscape = (body: string, landscape: boolean): string =>
  withProperty(body, "orientation", landscape ? "landscape" : null);

export const withSheets = (body: string, on: boolean): string =>
  withProperty(body, "sheets", on ? "on" : null);
