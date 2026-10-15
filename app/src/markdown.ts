export const INSIDE = "data-inside";

export const DOC = "tisty:doc/";

export const docLink = (id: string, title: string): string =>
  `[${title.replace(/([[\]\\])/g, "\\$1")}](${DOC}${id})`;

export const docCard = (id: string, title: string): string => `!${docLink(id, title)}`;

export const docOf = (target: string): string | null =>
  target.startsWith(DOC) ? target.slice(DOC.length) : null;

const ours = (target: string): boolean =>
  !/^[a-z][a-z0-9+.-]*:/i.test(target) &&
  !/^[a-z]:/i.test(target) &&
  !target.startsWith("/") &&
  !target.startsWith("\\\\") &&
  !target.startsWith("#");

const BARE = /\b(?:https?:\/\/|www\.)[^\s<>"]*[^\s<>".,;:!?)\]'*_]/gi;

// Notes were always written loosely, so a bare address still becomes a link here.
export const dressed = (html: string, steps: string[] = []): string => {
  const box = document.createElement("template");
  box.innerHTML = html;
  const all = box.content;
  for (const one of all.querySelectorAll("[data-at]")) one.removeAttribute("data-at");
  for (const card of all.querySelectorAll("a.card[data-doc]")) {
    const chip = document.createElement("img");
    chip.alt = card.textContent ?? "";
    chip.setAttribute(INSIDE, `${DOC}${card.getAttribute("data-doc")}`);
    card.replaceWith(chip);
  }
  for (const link of all.querySelectorAll("a[href]")) {
    const target = link.getAttribute("href") ?? "";
    // The core sends a scheme it will not follow as "#", which here is words and not a way out.
    if (target === "#") {
      link.replaceWith(...link.childNodes);
      continue;
    }
    if (ours(target)) link.setAttribute(INSIDE, target);
    if (docOf(target)) link.classList.add("paper");
  }
  for (const picture of all.querySelectorAll("img[src]")) {
    const target = picture.getAttribute("src") ?? "";
    if (!ours(target)) continue;
    picture.setAttribute(INSIDE, target);
    picture.setAttribute("src", "");
  }
  for (const ref of all.querySelectorAll("span.ref[data-ref]")) {
    const step = /^#(\d{1,3})$/.exec(ref.getAttribute("data-ref") ?? "");
    if (!step) continue;
    const at = Number(step[1]);
    ref.classList.add("step");
    ref.setAttribute("data-step", String(at));
    ref.textContent = steps[at - 1] ?? `#${at}`;
  }
  linked(all);
  return box.innerHTML;
};

const linked = (root: DocumentFragment) => {
  const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const loose: Text[] = [];
  for (let node = walk.nextNode(); node; node = walk.nextNode()) {
    if (!node.parentElement?.closest("a, code, pre")) loose.push(node as Text);
  }
  for (const node of loose) {
    const said = node.data;
    const pieces = document.createDocumentFragment();
    let from = 0;
    for (const hit of said.matchAll(BARE)) {
      const at = hit.index ?? 0;
      pieces.append(said.slice(from, at));
      const link = document.createElement("a");
      link.href = hit[0].startsWith("www.") ? `https://${hit[0]}` : hit[0];
      link.textContent = hit[0];
      pieces.append(link);
      from = at + hit[0].length;
    }
    if (from === 0) continue;
    pieces.append(said.slice(from));
    node.replaceWith(pieces);
  }
};
