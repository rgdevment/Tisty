import type { Turn } from "./side";

export const made = <K extends keyof HTMLElementTagNameMap>(
  page: Document,
  tag: K,
  className: string,
  said = "",
): HTMLElementTagNameMap[K] => {
  const one = page.createElement(tag);
  one.className = className;
  one.textContent = said;
  return one;
};

export const pressed = (
  page: Document,
  className: string,
  said: string,
  run: (at: HTMLElement) => void,
) => {
  const one = made(page, "button", className, said);
  one.type = "button";
  one.addEventListener("click", () => run(one));
  return one;
};

// A press that says what it does to a screen reader and on hover, where the eye sees a glyph.
export const tooled = (
  page: Document,
  className: string,
  said: string,
  run: (at: HTMLElement) => void,
  ...shown: (Node | string)[]
) => {
  const one = pressed(page, className, "", run);
  one.append(...shown);
  one.setAttribute("aria-label", said);
  one.title = said;
  return one;
};

export const toggle = (
  page: Document,
  turn: Turn,
  said: string,
  shown: Node | string,
  className = "side-tool",
) => {
  const one = tooled(page, className, said, () => turn.onTurn(!turn.on), shown);
  one.setAttribute("aria-pressed", String(turn.on));
  return one;
};

export type Line = {
  readonly key: string;
  readonly label: string;
  readonly gist: string;
  readonly tone?: string;
  /** Open when it is first drawn, until the reader chooses. */
  readonly open?: boolean;
  /** A second line under the gist, shown folded or not, with what it is said as a title. */
  readonly under?: { readonly text: string; readonly title: string };
};

export const rowed = (page: Document, line: Line, fill?: (into: HTMLElement) => void) => {
  const label = made(page, "strong", "side-row-label", line.label);
  const gist = made(page, "span", `side-gist${line.tone ? ` is-${line.tone}` : ""}`, line.gist);
  if (line.under) {
    const under = made(page, "code", "side-under", line.under.text);
    under.title = line.under.title;
    gist.append(under);
  }
  if (!fill) {
    const still = made(page, "div", "side-row");
    still.append(label, gist);
    return still;
  }
  const open = made(page, "details", "side-row");
  open.dataset.row = line.key;
  open.open = Boolean(line.open);
  const summary = made(page, "summary", "side-row-head");
  summary.append(label, gist);
  const body = made(page, "div", "side-row-body");
  fill(body);
  open.append(summary, body);
  return open;
};

export const listed = (
  page: Document,
  pairs: readonly (readonly [string, string | undefined])[],
) => {
  const list = made(page, "dl", "side-facts");
  for (const [term, said] of pairs) {
    if (!said) continue;
    const pair = made(page, "div", "side-fact");
    const value = made(page, "dd", "", said);
    value.title = said;
    pair.append(made(page, "dt", "", term), value);
    list.append(pair);
  }
  return list;
};

const COPIED_MS = 2000;

/** Copies the text, and the press says so for a moment before it offers again. */
export const copyFrom = (at: HTMLElement, text: string, offer: string, done: string) => {
  const view = at.ownerDocument.defaultView;
  view?.navigator.clipboard.writeText(text).then(
    () => {
      at.textContent = done;
      view.setTimeout(() => {
        at.textContent = offer;
      }, COPIED_MS);
    },
    () => undefined,
  );
};
