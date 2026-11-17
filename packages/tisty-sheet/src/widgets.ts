import { checkWidget, strayInWidget, WIDGET, widgetPage } from "@tisty/contracts/check";

export const SHORTEST = 40;
export const TALLEST = 2400;

export type Span = { left: number; right: number; view: number };

export const CLIMBS_AT_MOST = 3;
export const CLIMBS_WITHIN = 250;

export const tall = (height: number): number =>
  Math.min(TALLEST, Math.max(SHORTEST, Math.ceil(height)));

export type Lending = {
  retheme: () => void;
  shade: (dark: boolean | null) => void;
  drop: () => void;
};

export type WidgetWord = "widgetTooBig" | "widgetNot" | "widgetFrame";

export type Lender = {
  readonly say: (word: WidgetWord, named?: string) => string;
  readonly dark: () => boolean;
  readonly onTheme: (run: () => void) => () => void;
};

const WEIGHT_SHOWN = `${Math.round(WIDGET.bytes / 1000)} KB`;

export const notWidget = (source: string, say: Lender["say"]): string | null => {
  if (new Blob([source]).size > WIDGET.bytes) return say("widgetTooBig", WEIGHT_SHOWN);
  const wrong = [...new Set([...checkWidget(source), ...strayInWidget(source)])];
  return wrong.length ? say("widgetNot", wrong.slice(0, 3).join(", ")) : null;
};

const framed = (page: Document, sandbox: string, dark: boolean, title: string) => {
  const frame = page.createElement("iframe");
  frame.className = "lit-widget";
  frame.setAttribute("sandbox", sandbox);
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.title = title;
  frame.style.height = `${SHORTEST}px`;
  frame.style.colorScheme = dark ? "dark" : "light";
  return frame;
};

const sizing = (frame: HTMLIFrameElement) => {
  let climbed = 0;
  let stride = 0;
  let climbedAt = Number.NEGATIVE_INFINITY;
  return (height: number) => {
    const next = tall(height);
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
  };
};

const refused = (drawn: HTMLElement, why: string) => {
  const said = drawn.ownerDocument.createElement("p");
  said.className = "lit-refused";
  said.textContent = why;
  drawn.replaceChildren(said);
};

const MEASURED_AT_MOST = 3000;

const spanOf = (inside: Document): Span | null => {
  const view = inside.defaultView?.innerWidth ?? 0;
  let left = Number.POSITIVE_INFINITY;
  let right = 0;
  let seen = 0;
  for (const one of inside.body.querySelectorAll("*")) {
    seen += 1;
    if (seen > MEASURED_AT_MOST) break;
    const at = one.getBoundingClientRect();
    const fills = at.left <= 0.5 && at.right >= view - 0.5;
    if (!at.width || !at.height || fills) continue;
    if (inside.defaultView?.getComputedStyle(one).position === "fixed") continue;
    left = Math.min(left, at.left);
    right = Math.max(right, at.right);
  }
  return right > 0 ? { left: Math.max(0, Math.round(left)), right: Math.ceil(right), view } : null;
};

const NOTHING_LENT: Lending = { retheme: () => {}, shade: () => {}, drop: () => {} };

export const lend = (
  drawn: HTMLElement,
  source: string,
  lender: Lender,
  onSpan?: (span: Span) => void,
): Lending => {
  const why = notWidget(source, lender.say);
  if (why) {
    refused(drawn, why);
    return NOTHING_LENT;
  }
  let forced: boolean | null = null;
  let seen: ResizeObserver | null = null;
  const shaded = () => forced ?? lender.dark();
  const frame = framed(
    drawn.ownerDocument,
    "allow-same-origin",
    shaded(),
    lender.say("widgetFrame"),
  );
  const grow = sizing(frame);
  const retheme = () => {
    frame.style.colorScheme = shaded() ? "dark" : "light";
    frame.contentDocument?.documentElement.classList.toggle("dark", shaded());
  };
  frame.addEventListener("load", () => {
    const inside = frame.contentDocument;
    const box = inside?.querySelector("main.w");
    if (!inside || !box) return;
    const tell = () => {
      grow(Math.max(box.scrollHeight, box.getBoundingClientRect().height));
      const span = onSpan && spanOf(inside);
      if (span) onSpan?.(span);
    };
    retheme();
    seen?.disconnect();
    seen = new ResizeObserver(tell);
    seen.observe(box);
    tell();
  });
  frame.srcdoc = widgetPage(source, shaded());
  drawn.replaceChildren(frame);
  const unfollow = lender.onTheme(retheme);
  return {
    retheme,
    shade: (dark) => {
      forced = dark;
      retheme();
    },
    drop: () => {
      unfollow();
      seen?.disconnect();
      seen = null;
    },
  };
};
