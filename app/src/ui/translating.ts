import { HUES } from "./paging";

export const PENS = ["yellow", ...HUES];

export type Translated = {
  readonly shown: string;
  readonly stored: (text: string) => string;
};

const FENCE = /^(?:\s*>)*\s*(```|~~~)/;

// Code keeps its words as written: a fence or a span is never translated either way.
const prose = (text: string, change: (words: string) => string): string => {
  let out = "";
  let fenced: string | null = null;
  for (const line of text.split(/(?<=\n)/)) {
    const fence = FENCE.exec(line)?.[1];
    if (fenced) {
      out += line;
      if (fence === fenced) fenced = null;
      continue;
    }
    if (fence) {
      fenced = fence;
      out += line;
      continue;
    }
    out += spans(line, change);
  }
  return out;
};

const spans = (line: string, change: (words: string) => string): string => {
  let out = "";
  let at = 0;
  let from = 0;
  while (at < line.length) {
    if (line[at] !== "`") {
      at += 1;
      continue;
    }
    const run = /^`+/.exec(line.slice(at))?.[0] ?? "`";
    const shut = line.indexOf(run, at + run.length);
    if (shut < 0) {
      at += run.length;
      continue;
    }
    out += change(line.slice(from, at)) + line.slice(at, shut + run.length);
    at = shut + run.length;
    from = at;
  }
  return out + change(line.slice(from));
};

const toTisty = (words: string): string =>
  words
    .replace(/<u>(.+?)<\/u>/g, "++$1++")
    .replace(/<mark>(.+?)<\/mark>/g, "==$1==")
    .replace(/<mark data-pen="([a-z]+)">(.+?)<\/mark>/g, (all, pen: string, said: string) => {
      if (pen === "yellow") return `==${said}==`;
      return PENS.includes(pen) ? `==${said}=={${pen}}` : all;
    });

const toDisk = (words: string): string =>
  words
    .replace(/(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/gu, "<u>$1</u>")
    .replace(/==(?=\S)((?:(?!==).)+?)(?<=\S)==\{([a-z]+)\}/g, (all, said: string, pen: string) => {
      if (pen === "yellow") return `==${said}==`;
      return PENS.includes(pen) ? `<mark data-pen="${pen}">${said}</mark>` : all;
    });

// The yellow pen has three spellings on disk and one in Tisty: they all read the same.
const yellowed = (text: string): string =>
  prose(text, (words) => words.replace(/<mark(?: data-pen="yellow")?>(.+?)<\/mark>/g, "==$1=="));

export const translated = (stored: string): Translated => {
  const shown = prose(stored, toTisty);
  const back = (text: string) => prose(text, toDisk);
  if (back(shown) !== yellowed(stored)) return { shown: stored, stored: (text) => text };
  return { shown, stored: back };
};
