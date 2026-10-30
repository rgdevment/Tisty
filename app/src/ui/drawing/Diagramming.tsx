import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { ask } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { asRead, kindName, kindOf } from "tisty-diagrams/read";
import { DiagramStudio } from "tisty-diagrams/studio";
import { drawing } from "tisty-editor/read";
import type { Viewed } from "../../core";
import {
  diagramAgain,
  diagramLeave,
  diagramOrder,
  diagramSave,
  diagramUnsaved,
} from "../../drawing";
import { fill, locale, t, type Word } from "../../locales";
import { saidPlainly } from "../../refusal";
import Menu from "../Menu";
import { darkNow, followTheme } from "../theming";
import { button, tinted } from "../viewer/Glyphs";
import TitleBar from "../viewer/TitleBar";
import WindowChrome from "../WindowChrome";
import { exportChoices, fileName } from "./exporting";

const useDark = () => {
  const [dark, setDark] = useState(darkNow);
  useEffect(() => followTheme(() => setDark(darkNow())), []);
  return dark;
};

// One object for the life of the window, so the studio's hands stay the same between renders.
const LOOKS = { button };

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
  const [exporting, setExporting] = useState<{ x: number; y: number } | null>(null);
  const dark = useDark();
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
            ? `${kindName(kindOf(text), locale())} · ${paper.place}`
            : kindName(kindOf(text), locale())
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
            setExporting({ x: box.left, y: box.bottom + 4 });
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
            // Blocks written above it since it opened change which diagram it is.
            name: () =>
              diagramOrder()
                .catch((problem) => {
                  setNote(saidPlainly(problem));
                  return 0;
                })
                .then((nth) => fileName(named, nth)),
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
      <DiagramStudio
        text={text}
        onText={setText}
        dark={dark}
        locale={locale()}
        draw={drawing}
        copy={writeText}
        looks={LOOKS}
      />
    </main>
  );
}
