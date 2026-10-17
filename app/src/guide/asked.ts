import { EditorView } from "@codemirror/view";
import { locale } from "../locales";
import { kindOf, slugOf } from "./topicOf";

// The block being written lives inside the page's frame, not in the window that heard the key.
export const askedHere = (): string | undefined => {
  for (const frame of document.querySelectorAll("iframe")) {
    let page: Document | null = null;
    try {
      page = frame.contentDocument;
    } catch {
      continue;
    }
    const at = page?.querySelector<HTMLElement>(".editing .cm-editor");
    const view = at ? EditorView.findFromDOM(at) : null;
    if (!view) continue;
    const { state } = view;
    const kind = kindOf(state.doc.toString(), state.doc.lineAt(state.selection.main.head).text);
    return kind ? slugOf(kind, locale().toLowerCase().startsWith("es")) : undefined;
  }
  return undefined;
};
