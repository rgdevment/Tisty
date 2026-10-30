import { fields, frontLines, keyed, laidIn, type Paper } from "tisty-editor/read";

export const unfronted = (body: string): string => {
  const held = frontLines(body);
  if (!held) return body;
  return body.replace(/^﻿/, "").split("\n").slice(held.length).join("\n");
};

export const frontTags = (body: string): string[] =>
  fields(body)
    .filter((one) => /^tags?$/i.test(one.key))
    .flatMap((one) => one.values.flatMap((value) => value.split(/[, ]/)))
    .map((word) => word.replace(/^#+/, ""))
    .filter((word) => word !== "");

export const valuesIn = (body: string, key: string): string[] | undefined =>
  keyed(fields(body), key);

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
    const named = new RegExp(String.raw`^${key}\s*:`, "i");
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
