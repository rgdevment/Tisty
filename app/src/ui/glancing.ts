import { docGlance, type Glance } from "../citing";
import { DOC } from "../markdown";

const OPENS_AFTER = 450;
const SHUTS_AFTER = 200;
const MARGIN = 12;

const aimedAt = (link: Element | null): { file: string; heading?: string } | null => {
  if (!link || link.classList.contains("card")) return null;
  const href = link.getAttribute("href") ?? "";
  if (!href.startsWith(DOC)) return null;
  const [file, heading] = href.slice(DOC.length).split("#");
  return file ? { file, heading: heading || undefined } : null;
};

const drawn = (page: Document, link: Element, said: Glance): HTMLElement => {
  const box = page.createElement("aside");
  box.className = "glance";
  box.setAttribute("role", "tooltip");
  const title = page.createElement("p");
  title.className = "glance-title";
  title.textContent = said.heading ? `${said.title} › ${said.heading}` : said.title;
  const body = page.createElement("div");
  body.className = "glance-body";
  body.innerHTML = said.html;
  box.append(title, body);
  if (said.more) box.dataset.more = "";
  page.body.append(box);
  const view = page.defaultView;
  const at = link.getBoundingClientRect();
  const room = page.documentElement.clientWidth - box.offsetWidth - MARGIN;
  box.style.left = `${Math.max(MARGIN, Math.min(at.left, room)) + (view?.scrollX ?? 0)}px`;
  box.style.top = `${at.bottom + 6 + (view?.scrollY ?? 0)}px`;
  return box;
};

// Resting on a link to another document shows its section, or its first words, drawn by the core.
export const glancing = (page: Document, look = docGlance) => {
  const root = page.documentElement;
  if (root.dataset.glancing) return;
  root.dataset.glancing = "on";
  let opening: ReturnType<typeof setTimeout> | undefined;
  let shutting: ReturnType<typeof setTimeout> | undefined;
  let over: Element | null = null;
  let shown: HTMLElement | null = null;
  const shut = () => {
    clearTimeout(opening);
    shown?.remove();
    shown = null;
    over = null;
  };
  const within = (one: Element | null, target: EventTarget | null) =>
    Boolean(
      one && typeof (target as Node | null)?.nodeType === "number" && one.contains(target as Node),
    );
  const inside = (target: EventTarget | null) => within(shown, target);

  page.addEventListener("mouseover", (event) => {
    clearTimeout(shutting);
    if (inside(event.target)) return;
    const link = (event.target as Element | null)?.closest?.("a[href]") ?? null;
    const aimed = aimedAt(link);
    if (!aimed || link === over) return;
    shut();
    over = link;
    opening = setTimeout(() => {
      look(aimed.file, aimed.heading)
        .then((said) => {
          if (over === link && link?.isConnected) shown = drawn(page, link, said);
        })
        .catch(() => {});
    }, OPENS_AFTER);
  });
  page.addEventListener("mouseout", (event) => {
    if (!over) return;
    const into = event.relatedTarget;
    if (inside(into) || within(over, into)) return;
    clearTimeout(shutting);
    shutting = setTimeout(shut, SHUTS_AFTER);
  });
  for (const ends of ["mousedown", "keydown", "scroll"] as const)
    page.addEventListener(ends, (event) => !inside(event.target) && shut(), true);
};
