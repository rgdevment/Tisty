import { convertFileSrc } from "@tauri-apps/api/core";
import { openLink, widgetLend, widgetLendKept, widgetTakeBack } from "../core";
import { t } from "../locales";
import { DOC } from "../markdown";
import { saidPlainly } from "../refusal";

export const SHORTEST = 40;
export const TALLEST = 2400;

export type Asked = { type: "resize"; height: number } | { type: "open"; href: string };

export const heard = (data: unknown): Asked | null => {
  if (!data || typeof data !== "object") return null;
  const said = data as Record<string, unknown>;
  if (said.type === "resize" && typeof said.height === "number" && Number.isFinite(said.height))
    return { type: "resize", height: said.height };
  if (said.type === "open" && typeof said.href === "string")
    return { type: "open", href: said.href };
  return null;
};

export const openable = (href: string): "doc" | "web" | null => {
  const at = href.trim();
  if (at.startsWith(DOC) && at.length > DOC.length) return "doc";
  if (/^https?:\/\/[^\s]+$/i.test(at)) return "web";
  return null;
};

export const OPENS_EVERY = 1000;

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

export type Lending = { retheme: () => void; drop: () => void };

export const lend = (drawn: HTMLElement, source: string): Lending =>
  drawnFrom(drawn, () => widgetLend(source));

export const lendKept = (drawn: HTMLElement, reference: string): Lending =>
  drawnFrom(drawn, () => widgetLendKept(reference));

const drawnFrom = (drawn: HTMLElement, borrowed: () => Promise<string>): Lending => {
  let id: string | null = null;
  let gone = false;
  let opened = Number.NEGATIVE_INFINITY;
  const frame = document.createElement("iframe");
  frame.className = "lit-widget";
  frame.setAttribute("sandbox", "allow-scripts");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.title = t("widgetFrame");
  frame.style.height = `${SHORTEST}px`;
  frame.style.colorScheme = darkNow() ? "dark" : "light";

  const listening = (event: MessageEvent) => {
    if (event.source !== frame.contentWindow) return;
    const asked = heard(event.data);
    if (!asked) return;
    if (asked.type === "resize") {
      frame.style.height = `${tall(asked.height)}px`;
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
        widgetTakeBack(lent).catch(() => {});
        return;
      }
      id = lent;
      frame.src = `${convertFileSrc(lent, "widget")}${darkNow() ? "?dark=1" : ""}`;
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
    frame.contentWindow?.postMessage({ type: "theme", dark: darkNow() }, "*");
  };
  followed(retheme);

  return {
    retheme,
    drop: () => {
      gone = true;
      following.delete(retheme);
      window.removeEventListener("message", listening);
      if (id) widgetTakeBack(id).catch(() => {});
      id = null;
    },
  };
};
