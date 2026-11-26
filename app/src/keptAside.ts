import { stored } from "./stored";

const aside = stored<Record<string, string>>("tisty.aside", (said) =>
  Object.fromEntries(
    Object.entries(said).filter((one): one is [string, string] => typeof one[1] === "string"),
  ),
);

export const setAside = (id: string, words: string): void => {
  aside.put({ ...aside.current(), [id]: words });
};

export const asideWords = (id: string): string | undefined => aside.current()[id];

export const letAsideGo = (id: string): void => {
  if (asideWords(id) === undefined) return;
  const { [id]: _, ...rest } = aside.current();
  aside.put(rest);
};

export const useAside = (id: string): string | undefined => aside.use()[id];
