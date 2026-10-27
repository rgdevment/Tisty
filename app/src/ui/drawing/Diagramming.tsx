import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import type { Viewed } from "../../core";
import { diagramAgain, diagramLeave, diagramSave, diagramUnsaved } from "../../drawing";
import { fill, t, type Word } from "../../locales";
import { saidPlainly } from "../../refusal";
import { darkNow } from "../viewer/acting";
import { button, tinted } from "../viewer/Glyphs";
import TitleBar from "../viewer/TitleBar";
import WindowChrome from "../WindowChrome";
import Canvas from "./Canvas";
import { asRead, type DiagramKind, kindOf } from "./framing";
import IdeaMap from "./IdeaMap";
import IdeaPanel from "./IdeaPanel";
import { renamed, reshaped, withChild, without, withSibling } from "./minding";
import SchemaBoard from "./SchemaBoard";
import SchemaPanel from "./SchemaPanel";
import { useMind } from "./useMind";
import { useSchema } from "./useSchema";

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
  const dark = useDark();
  const map = useMind(text, setText);
  const scheme = useSchema(text, setText);
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
        <output className="text-[12px] text-faint">{t(stateOf(saving, unsaved))}</output>
        <button type="button" className={tinted} disabled={!unsaved || saving} onClick={save}>
          {fill("diagramSave", named)}
        </button>
      </TitleBar>
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
        {map ? (
          <IdeaMap
            mind={map.mind}
            chosen={map.chosen}
            fresh={t("diagramNewIdea")}
            onChoose={map.choose}
            onChange={map.change}
          />
        ) : scheme ? (
          <SchemaBoard
            schema={scheme.schema}
            picked={scheme.picked}
            fresh={t("schemaNewTable")}
            onPick={scheme.pick}
            onChange={scheme.change}
          />
        ) : (
          <Canvas source={text} dark={dark} />
        )}
        <aside
          aria-label={t("diagramProperties")}
          className="flex w-[340px] shrink-0 flex-col gap-2 border-l border-hair bg-panel p-4"
        >
          {map && (
            <IdeaPanel
              idea={map.idea}
              root={map.chosen.length === 0}
              onRename={(words) => map.change(renamed(map.mind, map.chosen, words))}
              onShape={(shape) => map.change(reshaped(map.mind, map.chosen, shape))}
              onChild={() => map.change(withChild(map.mind, map.chosen, t("diagramNewIdea")))}
              onSibling={() => map.change(withSibling(map.mind, map.chosen, t("diagramNewIdea")))}
              onRemove={() => map.change(without(map.mind, map.chosen))}
            />
          )}
          {scheme && (
            <div className="scroller max-h-[60%] shrink-0 overflow-auto border-b border-hair pb-3">
              <SchemaPanel schema={scheme.schema} picked={scheme.picked} onChange={scheme.change} />
            </div>
          )}
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
