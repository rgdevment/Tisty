import {
  type Asking,
  type Linkable,
  linkAsked,
  linksAsked,
  TAG_EDGE,
  TAG_TAIL,
  TAG_WORD,
} from "tisty-editor/edit";
import { t } from "../locales";

const TAGGED = new RegExp(`${TAG_EDGE}#${TAG_WORD}$`, "u");
const REST = new RegExp(`^${TAG_TAIL}*`, "u");
const SHOWN = 8;

const plain = (word: string) => word.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

export type Listed<T> = {
  readonly from: number;
  readonly to: number;
  readonly shown: readonly T[];
};

export type TagMatch = Listed<string>;

export const tagMatch = (text: string, at: number, tags: readonly string[]): TagMatch | null => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const line = text.slice(start, at);
  if ((line.match(/`/g)?.length ?? 0) % 2) return null;
  const found = TAGGED.exec(line);
  if (!found) return null;
  const word = found[0].slice(1);
  if (!/\p{L}/u.test(word)) return null;
  const key = plain(word);
  if (tags.includes(key)) return null;
  const shown = tags.filter((one) => one.startsWith(key)).slice(0, SHOWN);
  if (shown.length === 0) return null;
  const end = text.indexOf("\n", at);
  const rest = REST.exec(text.slice(at, end < 0 ? text.length : end))?.[0].length ?? 0;
  return { from: at - found[0].length, to: at + rest, shown };
};

export const linkMatch = (
  text: string,
  at: number,
  host: Pick<Asking, "linkable" | "headsOf">,
  ready: () => void,
): Listed<Linkable> | null => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const line = text.slice(start, at);
  if ((line.match(/`/g)?.length ?? 0) % 2) return null;
  const asked = linkAsked(line);
  if (!asked) return null;
  const shown = linksAsked(host, asked, ready);
  if (shown.length === 0) return null;
  const closed = text.slice(at, at + 2) === "]]";
  return { from: start + asked.from, to: closed ? at + 2 : at, shown };
};

export type CaretList<T> = {
  readonly offer: (match: Listed<T> | null) => void;
  readonly hide: () => void;
  readonly stop: () => void;
};

export type TagList = CaretList<string>;

export const tagList = (page: Document, pick: (match: TagMatch, tag: string) => void): TagList =>
  caretList(page, t("tagsInUse"), (tag) => ({ label: `#${tag}` }), pick, false);

export const linkList = (
  page: Document,
  pick: (match: Listed<Linkable>, one: Linkable) => void,
): CaretList<Linkable> => caretList(page, t("linkDocs"), (one) => one, pick, true);

const caretList = <T>(
  page: Document,
  named: string,
  said: (one: T) => { readonly label: string; readonly detail?: string },
  pick: (match: Listed<T>, one: T) => void,
  // A tag is kept as typed unless one is chosen; a link has nothing to keep, so Enter takes the first.
  firstLit: boolean,
): CaretList<T> => {
  const list = page.createElement("div");
  list.className = "tag-list";
  list.setAttribute("role", "listbox");
  list.setAttribute("aria-label", named);
  list.hidden = true;
  page.body.append(list);
  let match: Listed<T> | null = null;
  let lit = -1;

  const light = (at: number) => {
    lit = at;
    list.querySelectorAll("[role=option]").forEach((one, index) => {
      one.setAttribute("aria-selected", String(index === at));
    });
  };
  const hide = () => {
    list.hidden = true;
    match = null;
    lit = -1;
  };
  const chose = (one: T) => {
    const was = match;
    hide();
    if (was) pick(was, one);
  };

  // A caret right after a drawn tag measures nothing, so the tag before it stands in for it.
  const caretBox = (): { left: number; bottom: number } | null => {
    const held = page.getSelection();
    if (!held?.rangeCount) return null;
    const range = held.getRangeAt(0);
    const box =
      typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
    if (box && (box.height || box.left || box.bottom)) return box;
    const node = held.anchorNode;
    const before =
      node?.nodeType === Node.ELEMENT_NODE
        ? node.childNodes[held.anchorOffset - 1]
        : node?.previousSibling;
    const near = before?.nodeType === Node.ELEMENT_NODE ? (before as Element) : node?.parentElement;
    const rect = near?.getBoundingClientRect();
    return rect ? { left: before ? rect.right : rect.left, bottom: rect.bottom } : null;
  };

  const placed = () => {
    const box = caretBox();
    const view = page.defaultView;
    if (!box || !view) return hide();
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    list.style.top = `${(box.bottom + view.scrollY) / zoom + 6}px`;
    const widest = (view.innerWidth + view.scrollX) / zoom - list.offsetWidth - 4;
    list.style.left = `${Math.max(4, Math.min((box.left + view.scrollX) / zoom, widest))}px`;
  };

  const offer = (next: Listed<T> | null) => {
    if (!next) return hide();
    match = next;
    lit = -1;
    list.replaceChildren(
      ...next.shown.map((each) => {
        const one = page.createElement("div");
        one.setAttribute("role", "option");
        one.setAttribute("aria-selected", "false");
        const { label, detail } = said(each);
        one.textContent = label;
        if (detail) {
          const under = page.createElement("span");
          under.className = "tag-list-detail";
          under.textContent = detail;
          one.append(under);
        }
        one.addEventListener("mousedown", (event) => {
          event.preventDefault();
          chose(each);
        });
        return one;
      }),
    );
    list.hidden = false;
    if (firstLit) light(0);
    placed();
    page.defaultView?.requestAnimationFrame(placed);
  };
  const followed = () => {
    if (!list.hidden) placed();
  };
  page.addEventListener("selectionchange", followed);

  // Heard before the window's own keys, so Escape or an arrow meant for the list goes no further.
  const keyed = (event: KeyboardEvent) => {
    if (list.hidden || !match || event.ctrlKey || event.metaKey || event.altKey) return;
    const many = match.shown.length;
    const taken = () => {
      event.preventDefault();
      event.stopImmediatePropagation();
    };
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      taken();
      light(lit < 0 ? 0 : (lit + (event.key === "ArrowDown" ? 1 : many - 1)) % many);
      return;
    }
    if (event.key === "Tab" || (event.key === "Enter" && lit >= 0)) {
      taken();
      const one = match.shown[Math.max(lit, 0)];
      if (one !== undefined) chose(one);
      return;
    }
    if (event.key === "Escape") {
      taken();
      hide();
    }
  };
  page.addEventListener("keydown", keyed, true);

  return {
    offer,
    hide,
    stop: () => {
      page.removeEventListener("keydown", keyed, true);
      page.removeEventListener("selectionchange", followed);
      list.remove();
    },
  };
};
