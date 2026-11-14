import { TAG_EDGE, TAG_TAIL, TAG_WORD } from "tisty-editor/edit";
import { t } from "../locales";

const TAGGED = new RegExp(`${TAG_EDGE}#${TAG_WORD}$`, "u");
const REST = new RegExp(`^${TAG_TAIL}*`, "u");
const SHOWN = 8;

const plain = (word: string) => word.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

export type TagMatch = {
  readonly from: number;
  readonly to: number;
  readonly shown: readonly string[];
};

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

export type TagList = {
  readonly offer: (match: TagMatch | null) => void;
  readonly hide: () => void;
  readonly stop: () => void;
};

export const tagList = (page: Document, pick: (match: TagMatch, tag: string) => void): TagList => {
  const list = page.createElement("div");
  list.className = "tag-list";
  list.setAttribute("role", "listbox");
  list.setAttribute("aria-label", t("tagsInUse"));
  list.hidden = true;
  page.body.append(list);
  let match: TagMatch | null = null;
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
  const chose = (tag: string) => {
    const was = match;
    hide();
    if (was) pick(was, tag);
  };

  const placed = () => {
    const held = page.getSelection();
    const box = held?.rangeCount ? held.getRangeAt(0).getBoundingClientRect() : null;
    const view = page.defaultView;
    if (!box || !view) return hide();
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    list.style.top = `${(box.bottom + view.scrollY) / zoom + 6}px`;
    const widest = (view.innerWidth + view.scrollX) / zoom - list.offsetWidth - 4;
    list.style.left = `${Math.max(4, Math.min((box.left + view.scrollX) / zoom, widest))}px`;
  };

  const offer = (next: TagMatch | null) => {
    if (!next) return hide();
    match = next;
    lit = -1;
    list.replaceChildren(
      ...next.shown.map((tag) => {
        const one = page.createElement("div");
        one.setAttribute("role", "option");
        one.setAttribute("aria-selected", "false");
        one.textContent = `#${tag}`;
        one.addEventListener("mousedown", (event) => {
          event.preventDefault();
          chose(tag);
        });
        return one;
      }),
    );
    list.hidden = false;
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
      chose(match.shown[Math.max(lit, 0)] ?? "");
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
