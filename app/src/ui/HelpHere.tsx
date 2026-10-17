import { EditorView } from "@codemirror/view";
import { useRef } from "react";
import { kindOf, slugOf } from "../guide/topicOf";
import { locale, t } from "../locales";

export const HELP = "tisty:help";

export const askHelp = (topic?: string) =>
  window.dispatchEvent(new CustomEvent<string | undefined>(HELP, { detail: topic }));

// The block being written lives inside the page's frame, not in the window that holds the button.
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

export default function HelpHere() {
  const held = useRef<string | undefined>(undefined);
  return (
    <button
      type="button"
      aria-label={t("helpHere")}
      title={`${t("helpHere")} · F1`}
      onPointerDown={(event) => {
        event.preventDefault();
        held.current = askedHere();
      }}
      onPointerLeave={() => {
        held.current = undefined;
      }}
      onClick={() => {
        askHelp(held.current ?? askedHere());
        held.current = undefined;
      }}
      className="grid size-[18px] shrink-0 place-items-center rounded-full bg-accent text-[11px] leading-none font-bold text-white hover:brightness-110"
    >
      ?
    </button>
  );
}
