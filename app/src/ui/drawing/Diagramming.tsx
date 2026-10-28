import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import type { Viewed } from "../../core";
import {
  diagramAgain,
  diagramLeave,
  diagramOrder,
  diagramSave,
  diagramUnsaved,
} from "../../drawing";
import { fill, t, type Word } from "../../locales";
import { saidPlainly } from "../../refusal";
import Menu from "../Menu";
import { darkNow } from "../viewer/acting";
import { button, tinted } from "../viewer/Glyphs";
import TitleBar from "../viewer/TitleBar";
import WindowChrome from "../WindowChrome";
import Canvas from "./Canvas";
import { exportChoices, fileName } from "./exporting";
import { asRead, type DiagramKind, kindOf } from "./framing";
import { useEditor } from "./useEditor";

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

const stateOf = (saving: boolean, unsaved: boolean): Word => {
  if (saving) return "diagramSaving";
  return unsaved ? "diagramUnsaved" : "diagramSaved";
};

export default function Diagramming({
  paper,
  source,
}: Readonly<{ paper: Viewed; source: string }>) {
  const [text, setText] = useState(source);
  const [kept, setKept] = useState(() => asRead(source));
  const [saving, setSaving] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [moved, setMoved] = useState(false);
  // What an export said holds only while the text is the one it was made from.
  const [told, setTold] = useState<{ said: string; of: string } | null>(null);
  const [exporting, setExporting] = useState<{ x: number; y: number; nth: number } | null>(null);
  const dark = useDark();
  const editor = useEditor(text, setText);
  const unsaved = asRead(text) !== kept;
  const named = paper.title || t("untitledDoc");
  const warned = useRef(false);

  const failed = (error_: unknown) => {
    setMoved((error_ as { code?: string } | null)?.code === "diagramMoved");
    setNote(saidPlainly(error_));
  };

  const sent = (now: string) =>
    diagramSave(now).then(() => {
      setKept(asRead(now));
      setNote(null);
      setMoved(false);
    });

  const busy = (work: () => Promise<unknown>) => {
    setSaving(true);
    work()
      .catch(failed)
      .finally(() => setSaving(false));
  };

  const save = () => {
    if (unsaved && !saving) busy(() => sent(text));
  };

  const theirs = () =>
    busy(() =>
      diagramAgain().then((now) => {
        setText(now);
        setKept(asRead(now));
        setNote(null);
        setMoved(false);
      }),
    );

  const mine = () => busy(() => diagramAgain().then(() => sent(text)));

  useEffect(() => {
    diagramUnsaved(unsaved).catch(() => {});
  }, [unsaved]);

  // A dialog that cannot open must not trap the window: the next try to leave goes through.
  const leaving = async (said: Word): Promise<boolean> => {
    try {
      return await ask(t(said), { kind: "warning" });
    } catch {
      if (warned.current) return true;
      warned.current = true;
      setNote(t("diagramUnasked"));
      return false;
    }
  };

  const latest = useRef({ save, unsaved, leaving });
  latest.current = { save, unsaved, leaving };

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
    const here = getCurrentWindow();
    const closing = here.onCloseRequested(async (e) => {
      if (!latest.current.unsaved) return;
      if (!(await latest.current.leaving("diagramLeave"))) e.preventDefault();
    });
    const quitting = here.listen("diagramLeaving", async () => {
      if (await latest.current.leaving("diagramLeaveApp")) await diagramLeave();
    });
    return () => {
      closing.then((off) => off()).catch(() => {});
      quitting.then((off) => off()).catch(() => {});
    };
  }, []);

  return (
    <main className="relative flex h-full flex-col bg-desk font-sans">
      <WindowChrome flush />
      <TitleBar
        title={fill("diagramOf", named)}
        under={
          paper.place
            ? `${t(KIND_WORDS[kindOf(text)])} · ${paper.place}`
            : t(KIND_WORDS[kindOf(text)])
        }
      >
        {told?.of === text && <span className="text-[12px] text-accent">{told.said}</span>}
        <output className="text-[12px] text-faint">{t(stateOf(saving, unsaved))}</output>
        <button
          type="button"
          className={button}
          aria-haspopup="menu"
          onClick={(e) => {
            const box = e.currentTarget.getBoundingClientRect();
            const at = { x: box.left, y: box.bottom + 4 };
            // Blocks written above it since it opened change which diagram it is.
            diagramOrder()
              .then((nth) => setExporting({ ...at, nth }))
              .catch(() => setExporting({ ...at, nth: 0 }));
          }}
        >
          {`${t("diagramExport")} ▾`}
        </button>
        <button type="button" className={tinted} disabled={!unsaved || saving} onClick={save}>
          {fill("diagramSave", named)}
        </button>
      </TitleBar>
      {exporting && (
        <Menu
          at={exporting}
          label={t("diagramExport")}
          choices={exportChoices({
            source: text,
            name: fileName(named, exporting.nth),
            dark,
            said: (said) => setTold({ said, of: text }),
            failed: (problem) => setNote(saidPlainly(problem)),
          })}
          onClose={() => setExporting(null)}
        />
      )}
      {note && (
        <div className="flex shrink-0 flex-wrap items-center gap-2 px-4 py-1.5">
          <p className="m-0 text-[12.5px] text-urgent" role="alert">
            {note}
          </p>
          {moved && (
            <>
              <button type="button" className={button} disabled={saving} onClick={theirs}>
                {t("diagramTakeTheirs")}
              </button>
              <button type="button" className={button} disabled={saving} onClick={mine}>
                {t("diagramKeepMine")}
              </button>
            </>
          )}
        </div>
      )}
      <div className="flex min-h-0 flex-1">
        {editor?.board ?? <Canvas source={text} dark={dark} />}
        <aside
          aria-label={t("diagramProperties")}
          className="flex w-[340px] shrink-0 flex-col gap-2 border-l border-hair bg-panel p-4"
        >
          {editor?.panel}
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
