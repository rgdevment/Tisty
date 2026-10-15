import { GFM, parser } from "@lezer/markdown";
import { known } from "../glyphs";
import { HUES } from "./paging";

export const PENS = ["yellow", ...HUES];

export const PEN = /==(?=\S)((?:(?!==).)+?)(?<=\S)==(?:\{([a-z]+)\})?/u;
export const UNDER = /(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/u;
export const NAMED = /(?<![\p{L}\p{N}])::([a-z0-9-]+)(?: ([a-z]+))?::(?![\p{L}\p{N}])/u;

export const CODE = new Set(["InlineCode", "FencedCode", "CodeBlock"]);

export type Translated = {
  readonly shown: string;
  readonly stored: (text: string) => string;
  readonly active: boolean;
};

const reading = parser.configure(GFM);
const KEPT = /\]\((<attachments\/[^<>\n]+>|attachments\/[^\s()<>]+)/g;
const ICON = /<span data-ico="([a-z0-9-]+)"(?: data-hue="([a-z]+)")?>:([a-z0-9-]+):<\/span>/g;
const every = (one: RegExp) => new RegExp(one.source, "gu");

const prose = (text: string, change: (words: string) => string): string => {
  let out = "";
  let at = 0;
  reading.parse(text).iterate({
    enter: (node) => {
      if (!CODE.has(node.name)) return;
      out += change(text.slice(at, node.from)) + text.slice(node.from, node.to);
      at = node.to;
      return false;
    },
  });
  return out + change(text.slice(at));
};

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
    .replace(/<mark>(.+?)<\/mark>/g, "==$1=={yellow}")
    .replace(/<mark data-pen="([a-z]+)">(.+?)<\/mark>/g, (all, pen: string, said: string) =>
      pen !== "yellow" && PENS.includes(pen) ? `==${said}=={${pen}}` : all,
    )
    .replace(ICON, iconed)
    .replace(KEPT, (all, target: string) => {
      const name = shortOf(target);
      return kept.get(name) === target ? `](${name}` : all;
    });

const toDisk = (words: string, kept: Map<string, string>): string =>
  words
    .replace(every(UNDER), "<u>$1</u>")
    .replace(every(PEN), (all, said: string, pen: string | undefined) => {
      if (pen === undefined || !PENS.includes(pen)) return all;
      return pen === "yellow" ? `<mark>${said}</mark>` : `<mark data-pen="${pen}">${said}</mark>`;
    })
    .replace(every(NAMED), (all, name: string, hue: string | undefined) => {
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

export const translated = (stored: string): Translated => {
  const kept = keptIn(stored);
  const shown = prose(stored, (words) => toTisty(words, kept));
  const back = (text: string) => prose(text, (words) => toDisk(words, kept));
  if (back(shown) !== stored) return { shown: stored, stored: (text) => text, active: false };
  return { shown, stored: back, active: true };
};
