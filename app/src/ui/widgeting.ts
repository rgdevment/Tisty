import { convertFileSrc } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import type { Editor as Writing } from "@tiptap/core";
import { NodeSelection } from "@tiptap/pm/state";
import { type Borrowed, openLink, widgetLend, widgetLendKept, widgetTakeBack } from "../core";
import { t } from "../locales";
import { DOC } from "../markdown";
import { saidPlainly } from "../refusal";

export const SHORTEST = 40;
export const TALLEST = 2400;

export type Span = { left: number; right: number; view: number };

export type Asked =
  | { type: "resize"; height: number }
  | { type: "open"; href: string }
  | ({ type: "span" } & Span);

const counted = (one: unknown): one is number => typeof one === "number" && Number.isFinite(one);

export const heard = (data: unknown): Asked | null => {
  if (!data || typeof data !== "object") return null;
  const said = data as Record<string, unknown>;
  if (said.type === "resize" && typeof said.height === "number" && Number.isFinite(said.height))
    return { type: "resize", height: said.height };
  if (said.type === "open" && typeof said.href === "string")
    return { type: "open", href: said.href };
  if (said.type === "span" && counted(said.left) && counted(said.right) && counted(said.view))
    return { type: "span", left: said.left, right: said.right, view: said.view };
  return null;
};

export const openable = (href: string): "doc" | "web" | null => {
  const at = href.trim();
  if (at.startsWith(DOC) && at.length > DOC.length) return "doc";
  if (/^https?:\/\/[^\s]+$/i.test(at)) return "web";
  return null;
};

export const OPENS_EVERY = 1000;
export const CLIMBS_AT_MOST = 3;
export const CLIMBS_WITHIN = 250;

export const clickedInto = (frame: HTMLIFrameElement): boolean => {
  const activation = (navigator as { userActivation?: { isActive: boolean } }).userActivation;
  return Boolean(activation?.isActive) && document.activeElement === frame;
};

export const tall = (height: number): number =>
  Math.min(TALLEST, Math.max(SHORTEST, Math.ceil(height)));

const darkNow = () => document.documentElement.getAttribute("data-theme") === "dark";

const following = new Set<() => void>();
let watching: MutationObserver | null = null;

const followed = (retheme: () => void) => {
  following.add(retheme);
  if (watching) return;
  watching = new MutationObserver(() => {
    for (const one of following) one();
  });
  watching.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
};

export type Lending = {
  retheme: () => void;
  shade: (dark: boolean) => void;
  drop: () => void;
};

export const lend = (drawn: HTMLElement, source: string, onSpan?: (span: Span) => void): Lending =>
  drawnFrom(drawn, () => widgetLend(source).then((id) => ({ id, whole: false })), onSpan);

export const lendKept = (
  drawn: HTMLElement,
  reference: string,
  onSpan?: (span: Span) => void,
): Lending => drawnFrom(drawn, () => widgetLendKept(reference), onSpan);

const drawnFrom = (
  drawn: HTMLElement,
  borrowed: () => Promise<Borrowed>,
  onSpan?: (span: Span) => void,
): Lending => {
  let id: string | null = null;
  let whole = false;
  let gone = false;
  let opened = Number.NEGATIVE_INFINITY;
  let climbed = 0;
  let stride = 0;
  let climbedAt = Number.NEGATIVE_INFINITY;
  const frame = document.createElement("iframe");
  frame.className = "lit-widget";
  frame.setAttribute("sandbox", "allow-scripts");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.title = t("widgetFrame");
  frame.style.height = `${SHORTEST}px`;
  frame.style.colorScheme = darkNow() ? "dark" : "light";

  const listening = (event: MessageEvent) => {
    if (event.source !== frame.contentWindow || event.origin !== "null") return;
    const asked = heard(event.data);
    if (!asked) return;
    if (asked.type === "span") {
      onSpan?.(asked);
      return;
    }
    if (asked.type === "resize") {
      const next = tall(asked.height);
      const step = next - (Number.parseFloat(frame.style.height) || SHORTEST);
      if (step > 0) {
        const now = Date.now();
        climbed = step === stride && now - climbedAt < CLIMBS_WITHIN ? climbed + 1 : 1;
        stride = step;
        climbedAt = now;
        if (climbed >= CLIMBS_AT_MOST) return;
      } else {
        climbed = 0;
        stride = 0;
      }
      frame.style.height = `${next}px`;
      return;
    }
    const kind = openable(asked.href);
    if (!kind || !clickedInto(frame) || Date.now() - opened < OPENS_EVERY) return;
    opened = Date.now();
    if (kind === "web") openLink(asked.href.trim()).catch(() => {});
    if (kind === "doc")
      drawn.dispatchEvent(
        new CustomEvent("widgetdoc", {
          bubbles: true,
          detail: asked.href.trim().slice(DOC.length),
        }),
      );
  };
  window.addEventListener("message", listening);

  borrowed()
    .then((lent) => {
      if (gone) {
        widgetTakeBack(lent.id).catch(() => {});
        return;
      }
      id = lent.id;
      whole = lent.whole;
      frame.src = `${convertFileSrc(lent.id, "widget")}${darkNow() && !whole ? "?dark=1" : ""}`;
      drawn.replaceChildren(frame);
    })
    .catch((problem) => {
      if (gone) return;
      const said = document.createElement("p");
      said.className = "lit-refused";
      said.textContent = saidPlainly(problem);
      drawn.replaceChildren(said);
      drawn.dispatchEvent(new CustomEvent("widgetrefused", { bubbles: true }));
    });

  const retheme = () => {
    frame.style.colorScheme = darkNow() ? "dark" : "light";
    // A whole page may route by its own hash, so the theme never travels through its address.
    if (!frame.src || whole) return;
    frame.src = `${frame.src.split("#")[0]}#${darkNow() ? "dark" : "light"}`;
  };
  followed(retheme);

  return {
    retheme,
    shade: (dark) => {
      frame.style.colorScheme = dark ? "dark" : "light";
      if (!frame.src || whole) return;
      frame.src = `${frame.src.split("#")[0]}#${dark ? "dark" : "light"}`;
    },
    drop: () => {
      gone = true;
      following.delete(retheme);
      window.removeEventListener("message", listening);
      if (id) widgetTakeBack(id).catch(() => {});
      id = null;
    },
  };
};

export const besideWidget = (editor: Writing, back: boolean): boolean => {
  const { selection, doc } = editor.state;
  if (!selection.empty) return false;
  const at = selection.$from;
  if (at.depth === 0 || at.parent.type.name === "codeBlock") return false;
  if (back ? at.parentOffset !== 0 : at.parentOffset !== at.parent.content.size) return false;
  const edge = back ? at.before() : at.after();
  const near = back ? doc.resolve(edge).nodeBefore : doc.resolve(edge).nodeAfter;
  if (near?.type.name !== "codeBlock" || near.attrs.language !== "widget") return false;
  const start = back ? edge - near.nodeSize : edge;
  editor.view.dispatch(editor.state.tr.setSelection(NodeSelection.create(doc, start)));
  return true;
};

export type Pick = { label: string; run: () => void };

export const dropdown = (anchor: HTMLElement, picks: Pick[]): void => {
  const card = document.createElement("div");
  card.className = "widget-menu";
  card.setAttribute("role", "menu");
  const gone = () => {
    card.remove();
    document.removeEventListener("mousedown", away, true);
    document.removeEventListener("keydown", keyed, true);
  };
  const away = (e: MouseEvent) => {
    if (!card.contains(e.target as Node)) gone();
  };
  const keyed = (e: KeyboardEvent) => {
    if (e.key === "Escape") gone();
  };
  for (const one of picks) {
    const item = document.createElement("button");
    item.type = "button";
    item.setAttribute("role", "menuitem");
    item.textContent = one.label;
    item.addEventListener("click", (e) => {
      e.stopPropagation();
      gone();
      one.run();
    });
    card.append(item);
  }
  document.body.append(card);
  const box = anchor.getBoundingClientRect();
  const wide = card.getBoundingClientRect().width || 220;
  card.style.left = `${Math.max(6, Math.min(box.right - wide, window.innerWidth - wide - 6))}px`;
  card.style.top = `${box.bottom + 4}px`;
  document.addEventListener("mousedown", away, true);
  document.addEventListener("keydown", keyed, true);
  card.querySelector<HTMLButtonElement>("button")?.focus();
};

const ICONS = {
  apart:
    '<path d="M9.5 2.5H13.5V6.5M13.5 2.5L8 8"/><path d="M6.5 3.5H3.5A1 1 0 0 0 2.5 4.5V12.5A1 1 0 0 0 3.5 13.5H11.5A1 1 0 0 0 12.5 12.5V9.5"/>',
  more: '<circle cx="3.5" cy="8" r="1.1"/><circle cx="8" cy="8" r="1.1"/><circle cx="12.5" cy="8" r="1.1"/>',
  lock: '<rect x="3.5" y="7" width="9" height="6.5" rx="1.5"/><path d="M5.5 7V5.2a2.5 2.5 0 0 1 5 0V7"/>',
} as const;

export const iconed = (kind: keyof typeof ICONS, label: string, className: string) => {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.title = label;
  button.setAttribute("aria-label", label);
  button.innerHTML = `<svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICONS[kind]}</svg>`;
  return button;
};

export const locked = () => {
  const said = document.createElement("div");
  said.className = "lit-locked";
  said.setAttribute("contenteditable", "false");
  said.innerHTML = `<svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">${ICONS.lock}</svg>`;
  const why = document.createElement("span");
  const bold = document.createElement("b");
  bold.textContent = t("viewWidgetLocked");
  why.append(bold, ` ${t("viewWidgetLockedWhy")}`);
  said.append(why);
  return said;
};

export const failed = (problem: unknown) =>
  void message(saidPlainly(problem), { kind: "warning" }).catch(() => {});
