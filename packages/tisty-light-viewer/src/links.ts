import { follow, openLink } from "./host";

const BEHIND = "tisty.viewer.behind";
const ARRIVING_AT = "tisty.viewer.arriving";

const read = (): string[] => {
  try {
    return JSON.parse(sessionStorage.getItem(BEHIND) ?? "[]") as string[];
  } catch {
    return [];
  }
};

const kept = (key: string, value: string): void => {
  try {
    sessionStorage.setItem(key, value);
  } catch {
    return;
  }
};

export const canGoBack = (): boolean => read().length > 0;

export const goBack = async (): Promise<void> => {
  const behind = read();
  const last = behind.pop();
  if (!last) return;
  await follow(last);
  kept(BEHIND, JSON.stringify(behind));
  location.reload();
};

export const arrivingAt = (): string | null => {
  try {
    const anchor = sessionStorage.getItem(ARRIVING_AT);
    sessionStorage.removeItem(ARRIVING_AT);
    return anchor;
  } catch {
    return null;
  }
};

const toNeighbour = async (said: string, from: string): Promise<void> => {
  const [file, anchor] = said.split("#");
  await follow(file);
  kept(BEHIND, JSON.stringify([...read(), from]));
  if (anchor) kept(ARRIVING_AT, decodeURIComponent(anchor));
  location.reload();
};

export const linkPressed = (
  e: MouseEvent,
  from: string,
  fail: (problem: unknown) => void,
): void => {
  const link = (e.target as Element).closest<HTMLAnchorElement>("a[href]");
  if (!link) return;
  // A link left to the browser engine would take this window away from the document.
  e.preventDefault();
  const said = link.getAttribute("href") ?? "";
  if (said.startsWith("#")) {
    document.getElementById(decodeURIComponent(said.slice(1)))?.scrollIntoView({ block: "start" });
  } else if (/^(https?|mailto|tel):/i.test(said)) {
    void openLink(said).catch(fail);
  } else if (!/^[a-z][a-z0-9+.-]+:/i.test(said)) {
    void toNeighbour(said, from).catch(fail);
  }
};
