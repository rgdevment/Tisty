import { convertFileSrc } from "@tauri-apps/api/core";
import { darkNow, followTheme } from "tisty-reader/page";
import { type Borrowed, openLink, widgetLend, widgetLendKept, widgetTakeBack } from "../core";
import { t } from "../locales";
import { DOC } from "../markdown";
import { saidPlainly } from "../refusal";
import { drawn } from "./viewer/shapes";

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
  return Boolean(activation?.isActive) && frame.ownerDocument.activeElement === frame;
};

export const tall = (height: number): number =>
  Math.min(TALLEST, Math.max(SHORTEST, Math.ceil(height)));

export type Lending = {
  retheme: () => void;
  shade: (dark: boolean | null) => void;
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
  let forced: boolean | null = null;
  const shaded = () => forced ?? darkNow();
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
  // Inside a drawn page the widget speaks to that page's window, not the app's.
  const host = drawn.ownerDocument.defaultView ?? window;
  host.addEventListener("message", listening);

  borrowed()
    .then((lent) => {
      if (gone) {
        widgetTakeBack(lent.id).catch(() => {});
        return;
      }
      id = lent.id;
      whole = lent.whole;
      const asked = [shaded() && !whole ? "dark=1" : "", onSpan ? "span=1" : ""].filter(Boolean);
      frame.src = `${convertFileSrc(lent.id, "widget")}${asked.length ? `?${asked.join("&")}` : ""}`;
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
    frame.style.colorScheme = shaded() ? "dark" : "light";
    // A whole page may route by its own hash, so the theme never travels through its address.
    if (!frame.src || whole) return;
    frame.src = `${frame.src.split("#")[0]}#${shaded() ? "dark" : "light"}`;
  };
  const unfollow = followTheme(retheme);

  return {
    retheme,
    shade: (dark) => {
      forced = dark;
      retheme();
    },
    drop: () => {
      gone = true;
      unfollow();
      host.removeEventListener("message", listening);
      if (id) widgetTakeBack(id).catch(() => {});
      id = null;
    },
  };
};

export const locked = () => {
  const said = document.createElement("div");
  said.className = "lit-locked";
  said.setAttribute("contenteditable", "false");
  said.append(drawn("lock", 14));
  const why = document.createElement("span");
  const bold = document.createElement("b");
  bold.textContent = t("viewWidgetLocked");
  why.append(bold, ` ${t("viewWidgetLockedWhy")}`);
  said.append(why);
  return said;
};
