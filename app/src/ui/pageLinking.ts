// A link's address is not on the page it leads from, so it is asked for in a field of its own under the words.
export const addressAsked = (page: Document, near: DOMRect, said: string): Promise<string | null> =>
  new Promise((done) => {
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const field = page.createElement("input");
    field.type = "url";
    field.className = "page-address";
    field.placeholder = said;
    field.setAttribute("aria-label", said);
    field.style.top = `${(near.bottom + (view?.scrollY ?? 0)) / zoom + 6}px`;
    field.style.left = `${(near.left + (view?.scrollX ?? 0)) / zoom}px`;
    page.body.append(field);
    let over = false;
    const finish = (to: string | null) => {
      if (over) return;
      over = true;
      field.remove();
      done(to?.trim() || null);
    };
    field.addEventListener("keydown", (event) => {
      if (event.key === "Enter") finish(field.value);
      else if (event.key === "Escape") finish(null);
      else return;
      event.preventDefault();
      event.stopPropagation();
    });
    field.addEventListener("blur", () => finish(null));
    field.focus();
  });
