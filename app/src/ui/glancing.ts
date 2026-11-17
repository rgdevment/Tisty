import { docGlance, type Glance } from "../citing";
import { docAimed } from "../markdown";

const OPENS_AFTER = 450;
const SHUTS_AFTER = 200;
const MARGIN = 12;

const aimedAt = (link: Element | null) => {
  if (!link || link.classList.contains("card")) return null;
  const aimed = docAimed(link.getAttribute("href") ?? "");
  return aimed?.file ? aimed : null;
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
  // Its ids and places name the other document, and the page finds things by both.
  for (const one of body.querySelectorAll("[id], [data-at]")) {
    one.removeAttribute("id");
    one.removeAttribute("data-at");
  }
  box.append(title, body);
  if (said.more) box.dataset.more = "";
  page.body.append(box);
  const view = page.defaultView;
  const at = link.getBoundingClientRect();
  const room = page.documentElement.clientWidth - box.offsetWidth - MARGIN;
  const below = page.documentElement.getBoundingClientRect().bottom - at.bottom;
  // The frame is only as tall as the page, so a glance with no room below opens above the link.
  const top =
    below < box.offsetHeight + 6 ? Math.max(0, at.top - box.offsetHeight - 6) : at.bottom + 6;
  box.style.left = `${Math.max(MARGIN, Math.min(at.left, room)) + (view?.scrollX ?? 0)}px`;
  box.style.top = `${top + (view?.scrollY ?? 0)}px`;
  return box;
};

// Another document's section, drawn by the core, takes no clicks: its marks name that document's places.
export const glancing = (page: Document, look = docGlance) => {
  const root = page.documentElement;
  if (root.dataset.glancing) return;
  root.dataset.glancing = "on";
  let opening: ReturnType<typeof setTimeout> | undefined;
  let shutting: ReturnType<typeof setTimeout> | undefined;
  let over: Element | null = null;
  let shown: HTMLElement | null = null;
  let asked = 0;
  const shut = () => {
    asked += 1;
    clearTimeout(opening);
    shown?.remove();
    shown = null;
    over = null;
  };
  const within = (one: Element | null, target: EventTarget | null) =>
    Boolean(
      one && typeof (target as Node | null)?.nodeType === "number" && one.contains(target as Node),
    );

  page.addEventListener("mouseover", (event) => {
    clearTimeout(shutting);
    const link = (event.target as Element | null)?.closest?.("a[href]") ?? null;
    const aimed = aimedAt(link);
    if (!aimed || link === over) return;
    shut();
    over = link;
    const asking = ++asked;
    opening = setTimeout(() => {
      look(aimed.file, aimed.heading)
        .then((said) => {
          if (asking === asked && over === link && link?.isConnected) {
            shown?.remove();
            shown = drawn(page, link, said);
          }
        })
        .catch(() => {});
    }, OPENS_AFTER);
  });
  page.addEventListener("mouseout", (event) => {
    if (!over) return;
    const into = event.relatedTarget;
    if (within(over, into)) return;
    clearTimeout(shutting);
    shutting = setTimeout(shut, SHUTS_AFTER);
  });
  for (const ends of ["mousedown", "keydown", "scroll"] as const)
    page.addEventListener(ends, shut, true);
};
