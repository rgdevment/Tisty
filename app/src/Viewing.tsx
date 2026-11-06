import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { PAPERS } from "tisty-editor/read";
import {
  type DocFacts,
  opened,
  type Paper,
  type Showing,
  viewFacts,
  viewGo,
  viewShown,
} from "./core";
import { paperIn } from "./front";
import { adopt, t } from "./locales";
import { saidPlainly } from "./refusal";
import { fell } from "./Standing";
import Diagramming from "./ui/drawing/Diagramming";
import Menu from "./ui/Menu";
import { seenIn } from "./ui/printing";
import Quill from "./ui/Quill";
import { pageFor } from "./ui/sheets";
import { darkNow } from "./ui/theming";
import { leaveAsHtml, leaveAsWord } from "./ui/useLeaving";
import { type Acts, choicesFor, exportsFor, keysFor, PLAIN, ZOOMS } from "./ui/viewer/acting";
import { button } from "./ui/viewer/Glyphs";
import Header, { titled } from "./ui/viewer/Header";
import Preview from "./ui/viewer/Preview";
import type ReaderWindow from "./ui/viewer/Reader";
import Stage, { type Preset } from "./ui/viewer/Stage";
import usePdf from "./ui/viewer/usePdf";
import { useSheets } from "./ui/viewer/useSheets";
import WindowChrome from "./ui/WindowChrome";

const KEPT = "tisty.viewer";
export const SIDE_FROM = 1000;

// The index opens only where it leaves the page its full width.
const remembered = (): { side: boolean; leaf: Paper } => {
  const side = window.innerWidth >= SIDE_FROM;
  try {
    const said = JSON.parse(localStorage.getItem(KEPT) ?? "{}") as Record<string, unknown>;
    const leaf = PAPERS.includes(said.leaf as Paper) ? said.leaf : "a4";
    return { side, leaf: leaf as Paper };
  } catch {
    return { side, leaf: "a4" };
  }
};

const useReader = (wanted: boolean) => {
  const [loaded, setLoaded] = useState<typeof ReaderWindow | null>(null);
  useEffect(() => {
    if (!wanted) return;
    let live = true;
    void import("./ui/viewer/Reader").catch(fell).then(({ default: ready }) => {
      if (live) setLoaded(() => ready);
    });
    return () => {
      live = false;
    };
  }, [wanted]);
  return loaded;
};

// A diagram's window is its own thing: none of the reader's state is set up for it.
export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const Reader = useReader(shown?.kind === "doc");
  const [trouble, setTrouble] = useState<string | null>(null);
  // One look at a time: a second press, or a second mount in development, never fetches twice.
  const asking = useRef(false);
  const look = useCallback(() => {
    if (asking.current) return;
    asking.current = true;
    void viewShown()
      .then((one) => {
        setShown(one);
        setTrouble(null);
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => {
        asking.current = false;
      });
  }, []);
  useEffect(look, [look]);
  // Built hidden, so its first frame is this page rather than a bare window the system paints black.
  useEffect(
    () =>
      void getCurrentWindow()
        .show()
        .catch(() => {}),
    [],
  );

  if (shown?.kind === "diagram") return <Diagramming paper={shown.paper} source={shown.source} />;
  return <Reading shown={shown} Reader={Reader} trouble={trouble} look={look} />;
}

function Reading({
  shown,
  Reader,
  trouble,
  look,
}: {
  shown: Showing | null;
  Reader: typeof ReaderWindow | null;
  trouble: string | null;
  look: () => void;
}) {
  const [note, setNote] = useState<string | null>(null);
  // Each answer from the window, good or bad, replaces what the last one said.
  useEffect(() => setNote(trouble), [trouble, shown]);
  const [kept, setKept] = useState(remembered);
  const { side } = kept;
  const [raw, setRaw] = useState(false);
  const [zoom, setZoom] = useState(PLAIN);
  const [finding, setFinding] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number; only?: "export" } | null>(null);
  const [preset, setPreset] = useState<Preset>("fit");
  const [actual, setActual] = useState(false);
  const [dark, setDark] = useState<boolean | null>(null);
  const [again, setAgain] = useState(0);
  const [code, setCode] = useState(false);
  const [facts, setFacts] = useState<DocFacts | null>(null);

  useEffect(() => {
    try {
      localStorage.setItem(KEPT, JSON.stringify({ leaf: kept.leaf }));
    } catch {
      void 0;
    }
  }, [kept.leaf]);

  const reading = shown?.kind === "doc";

  useEffect(() => {
    setFacts(null);
    if (!reading) return;
    let gone = false;
    viewFacts()
      .then((now) => {
        if (!gone) setFacts(now);
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [reading, shown]);
  const body = reading ? shown.body : "";
  const paper = reading ? shown.paper : null;
  const shape = useSheets(paper?.file ?? null, body);
  const turned = shape.landscape;
  // The paper a document names is its own; the one kept here only fills in when it names none.
  const leaf = paperIn(body) ?? kept.leaf;
  useEffect(() => pageFor(leaf, turned), [leaf, turned]);
  const drawn = useRef<Window | null>(null);
  const fail = (problem: unknown) => setNote(saidPlainly(problem));
  const named = paper?.title ?? "";
  const pdf = usePdf({
    ready: reading,
    body,
    leaf,
    landscape: turned,
    paper,
    named,
    seen: () => seenIn(drawn.current?.document),
    onFail: fail,
  });

  useEffect(() => {
    if (shown?.kind !== "doc") return;
    // Focus moving into the drawn page and back is not the window coming back.
    let away = false;
    const left = () => {
      setTimeout(() => {
        away = !document.hasFocus();
      });
    };
    const back = () => {
      if (away) look();
      away = false;
    };
    window.addEventListener("blur", left);
    window.addEventListener("focus", back);
    return () => {
      window.removeEventListener("blur", left);
      window.removeEventListener("focus", back);
    };
  }, [shown?.kind, look]);

  const edit = () => {
    if (paper) void viewGo(paper.file).catch(fail);
  };

  const acts: Acts = {
    shown,
    body,
    dark,
    actual,
    fail,
    find: () => setFinding(true),
    zoom: (step) =>
      setZoom((at) => (step === 0 ? PLAIN : Math.max(0, Math.min(ZOOMS.length - 1, at + step)))),
    side: () => setKept((was) => ({ ...was, side: !was.side })),
    raw: () => setRaw((was) => !was),
    preview: () => void pdf.preview(),
    keep: () => void pdf.keep(),
    html: () => void leaveAsHtml([body], named || t("untitledDoc")).catch(fail),
    word: () => void leaveAsWord([body], named || t("untitledDoc")).catch(fail),
    edit,
    again: () => setAgain((n) => n + 1),
    code: () => setCode(true),
    shade: () => setDark((was) => !(was ?? darkNow())),
    fit: () => setActual((was) => !was),
    preset: setPreset,
    close: () => void getCurrentWindow().close(),
  };

  const keys = useRef(acts);
  keys.current = acts;
  const escaping = useRef(() => {});
  escaping.current = () => {
    if (pdf.seeing) pdf.shut();
    else if (code) setCode(false);
    else setFinding(false);
  };
  useEffect(() => {
    const heard = (e: KeyboardEvent) => {
      if (e.key === "Escape") return escaping.current();
      if (!(e.ctrlKey || e.metaKey)) return;
      const act = keysFor(keys.current, e);
      if (!act) return;
      e.preventDefault();
      act();
    };
    window.addEventListener("keydown", heard);
    return () => window.removeEventListener("keydown", heard);
  }, []);

  const { title, under } = titled(shown);

  return (
    <main className="relative flex h-full flex-col bg-desk font-sans">
      <WindowChrome flush />
      <Header
        shown={shown}
        side={side}
        raw={raw}
        preset={preset}
        menu={Boolean(menu)}
        onSide={acts.side}
        onRaw={setRaw}
        onEdit={edit}
        onPreset={setPreset}
        onAgain={acts.again}
        onMore={setMenu}
      />
      {note && (
        <p className="shrink-0 px-4 py-1.5 text-[12.5px] text-urgent" role="alert">
          {note}
        </p>
      )}
      {reading && Reader && (
        <Reader
          key={shown.paper.file}
          body={body}
          title={title}
          under={under}
          side={side}
          raw={raw}
          zoom={ZOOMS[zoom]}
          leaf={leaf}
          sheets={shape.sheets}
          landscape={turned}
          onPage={(page) => {
            drawn.current = page;
          }}
          facts={facts}
          finding={finding}
          onLeaf={(one) => setKept((was) => ({ ...was, leaf: one }))}
          onSheets={shape.onSheets}
          onLandscape={shape.onLandscape}
          onRaw={setRaw}
          onEdit={edit}
          onExport={(at) => setMenu({ ...at, only: "export" })}
          onFound={() => setFinding(false)}
          onDoc={(file) => void viewGo(file).catch(fail)}
          onOpen={(reference) => void opened(reference).catch(fail)}
        />
      )}
      {!trouble && (!shown || (reading && !Reader)) && <Quill said={t("viewOpening")} />}
      {shown?.kind === "widget" && (
        <Stage source={shown.source} preset={preset} actual={actual} dark={dark} again={again} />
      )}
      {code && shown?.kind === "widget" && (
        <div className="absolute inset-x-0 top-11 bottom-0 z-20 flex flex-col bg-desk">
          <div className="flex h-10 shrink-0 items-center border-b border-hair bg-panel pr-2.5 pl-4 text-[12.5px]">
            <b className="font-semibold">{t("viewCode")}</b>
            <button
              type="button"
              className={`ml-auto ${button} border-transparent text-soft`}
              onClick={() => setCode(false)}
            >
              {t("viewClose")}
            </button>
          </div>
          <pre className="scroller m-0 min-h-0 flex-1 whitespace-pre-wrap break-words px-6 py-5 font-mono text-[12.5px] leading-relaxed">
            {shown.source}
          </pre>
        </div>
      )}
      {pdf.seeing && (
        <Preview
          shown={pdf.seeing}
          leaf={leaf}
          stamping={pdf.stamping}
          coverable={Boolean(paper)}
          making={pdf.making}
          onStamp={pdf.setStamping}
          onKeep={() => void pdf.keep()}
          onClose={pdf.shut}
        />
      )}
      {menu && (
        <Menu
          at={menu}
          choices={menu.only ? exportsFor(acts) : choicesFor(acts)}
          label={t(menu.only ? "exportIt" : "viewMore")}
          onClose={() => setMenu(null)}
        />
      )}
    </main>
  );
}
