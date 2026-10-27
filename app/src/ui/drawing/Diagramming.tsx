import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { diagramSave, type Viewed } from "../../core";
import { fill, t, type Word } from "../../locales";
import { saidPlainly } from "../../refusal";
import { darkNow } from "../viewer/acting";
import { tinted } from "../viewer/Glyphs";
import WindowChrome, { clearOfChrome, onMac } from "../WindowChrome";
import { asRead, type DiagramKind, kindOf } from "./framing";
import Sheet from "./Sheet";

const KIND_WORDS: Record<DiagramKind, Word> = {
  mindmap: "diagramMindmap",
  erDiagram: "diagramSchema",
  flowchart: "diagramFlow",
  classDiagram: "diagramClasses",
  other: "diagramOther",
};

const useDark = () => {
  const [dark, setDark] = useState(darkNow);
  useEffect(() => {
    const watching = new MutationObserver(() => setDark(darkNow()));
    watching.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => watching.disconnect();
  }, []);
  return dark;
};

export default function Diagramming({ paper, source }: { paper: Viewed; source: string }) {
  const [text, setText] = useState(source);
  const [kept, setKept] = useState(source);
  const [saving, setSaving] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const dark = useDark();
  const unsaved = asRead(text) !== kept;
  const named = paper.title || t("untitledDoc");

  const save = () => {
    if (!unsaved || saving) return;
    setSaving(true);
    diagramSave(text)
      .then(() => {
        setKept(asRead(text));
        setNote(null);
      })
      .catch((problem) => setNote(saidPlainly(problem)))
      .finally(() => setSaving(false));
  };

  const latest = useRef({ save, unsaved });
  latest.current = { save, unsaved };

  useEffect(() => {
    const heard = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        latest.current.save();
      }
    };
    window.addEventListener("keydown", heard);
    return () => window.removeEventListener("keydown", heard);
  }, []);

  useEffect(() => {
    const leaving = getCurrentWindow().onCloseRequested(async (e) => {
      if (!latest.current.unsaved) return;
      if (!(await ask(t("diagramLeave"), { kind: "warning" }))) e.preventDefault();
    });
    return () => {
      leaving.then((off) => off()).catch(() => {});
    };
  }, []);

  return (
    <main className="relative flex h-full flex-col bg-desk font-sans">
      <WindowChrome flush />
      <header
        data-tauri-drag-region
        className={`flex h-11 shrink-0 items-center gap-2.5 border-b border-hair bg-panel ${onMac ? "pl-20" : "pl-3.5"} ${clearOfChrome}`}
      >
        <div data-tauri-drag-region className="flex min-w-0 flex-1 flex-col leading-tight">
          <b data-tauri-drag-region className="truncate text-[13px] font-semibold">
            {fill("diagramOf", named)}
          </b>
          <small data-tauri-drag-region className="mt-0.5 truncate text-[11.5px] text-faint">
            {t(KIND_WORDS[kindOf(text)])}
            {paper.place ? ` · ${paper.place}` : ""}
          </small>
        </div>
        <span className="text-[12px] text-faint" role="status">
          {saving ? t("diagramSaving") : unsaved ? t("diagramUnsaved") : t("diagramSaved")}
        </span>
        <button type="button" className={tinted} disabled={!unsaved || saving} onClick={save}>
          {fill("diagramSave", named)}
        </button>
      </header>
      {note && (
        <p className="shrink-0 px-4 py-1.5 text-[12.5px] text-urgent" role="alert">
          {note}
        </p>
      )}
      <div className="flex min-h-0 flex-1">
        <Sheet source={text} dark={dark} />
        <aside
          aria-label={t("diagramProperties")}
          className="flex w-[340px] shrink-0 flex-col gap-2 border-l border-hair bg-panel p-4"
        >
          <div className="flex items-center">
            <label htmlFor="diagram-text" className="text-[12px] font-semibold text-soft">
              {t("diagramText")}
            </label>
            <span className="ml-auto text-[11px] text-accent">{t("diagramLive")}</span>
          </div>
          <textarea
            id="diagram-text"
            spellCheck={false}
            value={text}
            onChange={(e) => setText(e.target.value)}
            className="scroller min-h-0 flex-1 resize-none rounded-md border border-line bg-desk p-3 font-mono text-[12.5px] leading-relaxed text-ink outline-none focus:border-accent"
          />
          <small className="text-[12px] leading-snug text-faint">{t("diagramKeptAsWritten")}</small>
        </aside>
      </div>
    </main>
  );
}
