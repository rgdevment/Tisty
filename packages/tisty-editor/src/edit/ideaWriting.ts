import type { EditorView } from "@codemirror/view";

const HEAD = /^(\s*>\s*\[!idea\][^\n]*)(?:\n|$)/i;

const wordsOf = (source: string): string =>
  source
    .replace(HEAD, "")
    .split("\n")
    .map((line) => line.replace(/^\s*>\s?/, ""))
    .join("\n");

const written = (head: string, words: string): string =>
  [head, ...words.split("\n").map((line) => (line ? `> ${line}` : ">"))].join("\n");

export const ideaEdited = (
  view: EditorView,
  note: HTMLElement,
  source: string,
  from: number,
): boolean => {
  const head = HEAD.exec(source)?.[1];
  if (head === undefined || note.querySelector(".cm-idea-words")) return false;
  const page = note.ownerDocument;
  const was = wordsOf(source);
  const shown = Array.from(note.childNodes);
  const kept = Array.from(note.querySelectorAll(":scope > .sticky-pin"));
  const box = page.createElement("div");
  box.className = "cm-idea-words";
  box.contentEditable = "plaintext-only";
  box.textContent = was;
  note.replaceChildren(...kept, box);
  box.focus();
  page.getSelection()?.selectAllChildren(box);
  page.getSelection()?.collapseToEnd();
  let done = false;
  const finish = (keep: boolean) => {
    if (done) return;
    done = true;
    const words = (box.innerText ?? box.textContent ?? "").replace(/\n+$/, "");
    if (!keep || words === was) {
      if (note.isConnected) note.replaceChildren(...shown);
    } else {
      view.dispatch({
        changes: { from, to: from + source.length, insert: written(head, words) },
        userEvent: "input.idea",
      });
    }
    view.focus();
  };
  box.addEventListener("keydown", (event) => {
    if (event.key === "Escape") finish(false);
    else if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) finish(true);
    else return;
    event.preventDefault();
    event.stopPropagation();
  });
  box.addEventListener("blur", () => finish(true));
  return true;
};
