// What the page does not show, as a link's address or a section's hint, is asked for in a field of its own under it.
export const fieldAsked = (
  page: Document,
  near: DOMRect,
  said: string,
  { value = "", kind = "text" }: { readonly value?: string; readonly kind?: "text" | "url" } = {},
): Promise<string | null> =>
  new Promise((done) => {
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const field = page.createElement("input");
    field.type = kind;
    field.value = value;
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
    field.select();
  });
