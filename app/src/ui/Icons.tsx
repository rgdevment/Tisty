import { useEffect, useState } from "react";
import { families as readFamilies, icons as readIcons } from "../core";
import { alsoNamed } from "../synonyms";

export interface Family {
  name: string;
  icons: string[];
}

interface Catalogue {
  all: string[];
  families: Family[];
}

const NONE: Catalogue = { all: [], families: [] };
let held: Catalogue | null = null;

const cut = (all: string[], cuts: [string, number][] | null): Family[] => {
  if (!cuts?.length) return [];
  const out: Family[] = [];
  let at = 0;
  for (const [name, many] of cuts) {
    out.push({ name, icons: all.slice(at, at + many) });
    at += many;
  }
  return at === all.length ? out : [];
};

export function useCatalogue(): Catalogue {
  const [all, setAll] = useState<Catalogue>(held ?? NONE);

  useEffect(() => {
    if (held) return;
    Promise.all([readIcons(), readFamilies().catch(() => null)])
      .then(([found, cuts]) => {
        held = { all: found, families: cut(found, cuts) };
        setAll(held);
      })
      .catch(() => {});
  }, []);

  return all;
}

/// What another program shows where Tisty draws the glyph.
export const spared = (key: string): string => `:${key}:`;

export const sifted = (all: string[], word: string): string[] => {
  const said = word.trim().toLowerCase();
  if (!said) return all;
  const also = new Set(alsoNamed(said));
  return all.filter((key) => key.includes(said) || also.has(key));
};
