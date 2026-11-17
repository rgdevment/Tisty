import { follow, openAttached, openLink, showDoc, showTask } from "./host";

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

const DOC = "tisty:doc/";
const TASK = "tisty:task/";

// A document of the person's own names other documents, tasks and attachments, all kept in Tisty.
const inTisty = (pressed: Element, said: string): Promise<void> | null => {
  const named = pressed.closest<HTMLElement>("[data-doc], [data-task]");
  if (named?.dataset.doc) return showDoc(named.dataset.doc);
  if (named?.dataset.task) return showTask(named.dataset.task);
  if (said.startsWith(DOC)) return showDoc(said.slice(DOC.length));
  if (said.startsWith(TASK)) return showTask(said.slice(TASK.length));
  if (said && !/^[a-z][a-z0-9+.-]+:/i.test(said) && !said.startsWith("#")) {
    return openAttached(said);
  }
  return null;
};

export const linkPressed = (
  e: MouseEvent,
  from: { readonly path: string; readonly own: boolean },
  fail: (problem: unknown) => void,
): void => {
  const pressed = e.target as Element;
  const link = pressed.closest<HTMLAnchorElement>("a[href], a[data-doc], a[data-task]");
  const card = pressed.closest(".doc-card[data-doc]");
  if (!link && !card) return;
  // A link left to the browser engine would take this window away from the document.
  e.preventDefault();
  const said = link?.getAttribute("href") ?? "";
  const kept = from.own ? inTisty(pressed, said) : null;
  if (kept) {
    void kept.catch(fail);
  } else if (said.startsWith("#")) {
    document.getElementById(decodeURIComponent(said.slice(1)))?.scrollIntoView({ block: "start" });
  } else if (/^(https?|mailto|tel):/i.test(said)) {
    void openLink(said).catch(fail);
  } else if (!from.own && said && !/^[a-z][a-z0-9+.-]+:/i.test(said)) {
    void toNeighbour(said, from.path).catch(fail);
  }
};
