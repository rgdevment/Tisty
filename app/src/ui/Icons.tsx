import { useEffect, useState } from "react";
import { icons as readIcons } from "../core";

let held: string[] | null = null;

export function useIcons() {
  const [all, setAll] = useState<string[]>(held ?? []);

  useEffect(() => {
    if (held) return;
    readIcons()
      .then((found) => {
        held = found;
        setAll(found);
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
  return all.filter((key) => key.includes(said));
};
