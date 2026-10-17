const SHOWN_AT_MOST = 60;

// A document's stages are counted as they are read, so #2 is the second box a reader meets.
export const staged = (page: Document) => {
  const stages = Array.from(page.querySelectorAll<HTMLElement>(".page li.task"));
  stages.forEach((one, at) => {
    one.id ||= `stage-${at + 1}`;
  });
  for (const ref of page.querySelectorAll<HTMLElement>('.page .ref[data-ref^="#"]')) {
    const asked = Number(ref.dataset.ref?.slice(1));
    const stage = Number.isInteger(asked) && asked > 0 ? stages[asked - 1] : undefined;
    if (!stage) continue;
    const words = (stage.textContent ?? "").replace(/\s+/g, " ").trim();
    ref.textContent =
      words.length > SHOWN_AT_MOST ? `${words.slice(0, SHOWN_AT_MOST - 1)}…` : words;
    ref.title = `#${asked}`;
    ref.classList.add("is-stage");
    ref.dataset.stage = stage.id;
  }
};
