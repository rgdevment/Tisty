import { known } from "../glyphs";
import { HUES } from "./paging";

export const PENS = ["yellow", ...HUES];

export type Translated = {
  readonly shown: string;
  readonly stored: (text: string) => string;
};

const FENCE = /^(?:\s*>)*\s*(```|~~~)/;
const KEPT = /\]\((<attachments\/[^<>\n]+>|attachments\/[^\s()<>]+)/g;
const ICON = /<span data-ico="([a-z0-9-]+)"(?: data-hue="([a-z]+)")?>:([a-z0-9-]+):<\/span>/g;
const NAMED = /(?<![\p{L}\p{N}])::([a-z0-9-]+)(?: ([a-z]+))?::(?![\p{L}\p{N}])/gu;

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

// An attachment is shown by the name it was given; the folder and the print Tisty adds stay on disk.
const shortOf = (target: string): string => {
  const path = target.replace(/^<|>$/g, "");
  const file = path.slice(path.lastIndexOf("/") + 1);
  return file.replace(/-[0-9a-f]{8}(?=\.[^.]+$)/, "");
};

const iconed = (all: string, name: string, hue: string | undefined, said: string): string => {
  if (name !== said) return all;
  if (hue === undefined) return `::${name}::`;
  return HUES.includes(hue) ? `::${name} ${hue}::` : all;
};

const toTisty = (words: string, kept: Map<string, string>): string =>
  words
    .replace(/<u>(.+?)<\/u>/g, "++$1++")
    .replace(/<mark>(.+?)<\/mark>/g, "==$1==")
    .replace(/<mark data-pen="([a-z]+)">(.+?)<\/mark>/g, (all, pen: string, said: string) => {
      if (pen === "yellow") return `==${said}==`;
      return PENS.includes(pen) ? `==${said}=={${pen}}` : all;
    })
    .replace(ICON, iconed)
    .replace(KEPT, (all, target: string) => {
      const name = shortOf(target);
      return kept.get(name) === target ? `](${name}` : all;
    });

const toDisk = (words: string, kept: Map<string, string>): string =>
  words
    .replace(/(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/gu, "<u>$1</u>")
    .replace(/==(?=\S)((?:(?!==).)+?)(?<=\S)==\{([a-z]+)\}/g, (all, said: string, pen: string) => {
      if (pen === "yellow") return `==${said}==`;
      return PENS.includes(pen) ? `<mark data-pen="${pen}">${said}</mark>` : all;
    })
    .replace(NAMED, (all, name: string, hue: string | undefined) => {
      if (!known(name) || (hue !== undefined && !HUES.includes(hue))) return all;
      const tint = hue === undefined ? "" : ` data-hue="${hue}"`;
      return `<span data-ico="${name}"${tint}>:${name}:</span>`;
    })
    .replace(/\]\(([^\s()<>]+)/g, (all, name: string) => {
      const target = kept.get(name);
      return target === undefined ? all : `](${target}`;
    });

// Two attachments a block calls by the same name could not be told apart again: such a name stays whole.
const keptIn = (text: string): Map<string, string> => {
  const kept = new Map<string, string>();
  const twice = new Set<string>();
  prose(text, (words) => {
    for (const one of words.matchAll(KEPT)) {
      const name = shortOf(one[1]);
      if (kept.has(name) && kept.get(name) !== one[1]) twice.add(name);
      kept.set(name, one[1]);
    }
    return words;
  });
  for (const name of twice) kept.delete(name);
  return kept;
};

// The yellow pen has three spellings on disk and one in Tisty: they all read the same.
const yellowed = (text: string): string =>
  prose(text, (words) => words.replace(/<mark(?: data-pen="yellow")?>(.+?)<\/mark>/g, "==$1=="));

export const translated = (stored: string): Translated => {
  const kept = keptIn(stored);
  const shown = prose(stored, (words) => toTisty(words, kept));
  const back = (text: string) => prose(text, (words) => toDisk(words, kept));
  if (back(shown) !== yellowed(stored)) return { shown: stored, stored: (text) => text };
  return { shown, stored: back };
};
