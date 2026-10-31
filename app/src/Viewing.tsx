import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useRef, useState } from "react";
import { PAPERS } from "tisty-editor/read";
import {
  type DocFacts,
  opened,
  type Paper,
  type Showing,
  viewFacts,
  viewGo,
  viewImport,
  viewShown,
} from "./core";
import { paperIn } from "./front";
import { leftBehind } from "./leftBehind";
import { adopt, fill, t } from "./locales";
import { plainly, saidPlainly } from "./refusal";
import Diagramming from "./ui/drawing/Diagramming";
import Menu from "./ui/Menu";
import { seenIn } from "./ui/printing";
import { pageFor } from "./ui/sheets";
import { darkNow } from "./ui/theming";
import { leaveAsHtml, leaveAsWord } from "./ui/useLeaving";
import { type Acts, choicesFor, exportsFor, keysFor, PLAIN, ZOOMS } from "./ui/viewer/acting";
import { button } from "./ui/viewer/Glyphs";
import Header, { titled } from "./ui/viewer/Header";
import Preview from "./ui/viewer/Preview";
import Problem, { type Trouble } from "./ui/viewer/Problem";
import Reader from "./ui/viewer/Reader";
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

// A share link Tisty could not open says why in its own words, and its link can still go to the browser.
const SHARED = "https://";

const troubleOf = (refusal: { code: string; name?: string }, path: string): Trouble =>
  refusal.code === "documentTooBig"
    ? { kind: "big", limit: refusal.name ?? "" }
    : path.startsWith(SHARED)
      ? { kind: "other", said: plainly(refusal) }
      : { kind: "unreadable" };

// A diagram's window is its own thing: none of the reader's state is set up for it.
export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const look = useCallback(
    () =>
      void viewShown()
        .then((one) => {
          setShown(one);
          setTrouble(null);
        })
        .catch((e) => setTrouble(saidPlainly(e))),
    [],
  );
  useEffect(look, [look]);

  if (shown?.kind === "diagram") return <Diagramming paper={shown.paper} source={shown.source} />;
  return <Reading shown={shown} trouble={trouble} look={look} />;
}

function Reading({
  shown,
  trouble,
  look,
}: {
  shown: Showing | null;
  trouble: string | null;
  look: () => void;
}) {
  const [note, setNote] = useState<string | null>(null);
  // Each answer from the window, good or bad, replaces what the last one said.
  useEffect(() => setNote(trouble), [trouble, shown]);
  const [band, setBand] = useState<string | null>(null);
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
  const [busy, setBusy] = useState(false);
  const [facts, setFacts] = useState<DocFacts | null>(null);

  useEffect(() => {
    try {
      localStorage.setItem(KEPT, JSON.stringify({ leaf: kept.leaf }));
    } catch {
      void 0;
    }
  }, [kept.leaf]);

  const reading = shown?.kind === "file" || shown?.kind === "doc";

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
  const which =
    shown?.kind === "doc" ? shown.paper.file : shown?.kind === "file" ? shown.path : null;
  const shape = useSheets(which, body);
  const turned = shape.landscape;
  // The paper a document names is its own; the one kept here only fills in when it names none.
  const leaf = paperIn(body) ?? kept.leaf;
  useEffect(() => pageFor(leaf, turned), [leaf, turned]);
  const paper = shown?.kind === "doc" ? shown.paper : null;
  const drawn = useRef<Window | null>(null);
  const fail = (problem: unknown) => setNote(saidPlainly(problem));
  const named = shown?.kind === "file" ? shown.name.replace(/\.[^.]+$/, "") : (paper?.title ?? "");
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

  const editIn = (open: boolean) => {
    if (paper) return void viewGo(paper.file).catch(fail);
    if (busy) return;
    const place = shown?.kind === "file" ? shown.place : "";
    setBusy(true);
    viewImport(open)
      .then((made) => {
        look();
        const left = leftBehind(made);
        const said = `${t(open ? "viewNowIn" : "viewSavedIn")} · ${fill("viewOriginal", place)}`;
        setBand(left ? `${said} · ${left}` : said);
      })
      .catch(fail)
      .finally(() => setBusy(false));
  };

  const acts: Acts = {
    shown,
    body,
    dark,
    actual,
    fail,
    look,
    find: () => setFinding(true),
    zoom: (step) =>
      setZoom((at) => (step === 0 ? PLAIN : Math.max(0, Math.min(ZOOMS.length - 1, at + step)))),
    side: () => setKept((was) => ({ ...was, side: !was.side })),
    raw: () => setRaw((was) => !was),
    preview: () => void pdf.preview(),
    keep: () => void pdf.keep(),
    html: () => void leaveAsHtml([body], named || t("untitledDoc")).catch(fail),
    word: () => void leaveAsWord([body], named || t("untitledDoc")).catch(fail),
    quietly: () => editIn(false),
    edit: () => editIn(true),
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
        busy={busy}
        preset={preset}
        menu={Boolean(menu)}
        onSide={acts.side}
        onRaw={setRaw}
        onEdit={() => editIn(true)}
        onPreset={setPreset}
        onAgain={acts.again}
        onMore={setMenu}
      />
      {band && (
        <div className="flex shrink-0 items-center gap-2.5 border-b border-hair bg-accent-soft px-4 py-[7px] text-[12.5px] text-soft">
          <span>{band}</span>
          <button
            type="button"
            className="ml-auto text-[12.5px] font-semibold text-accent"
            onClick={() => setBand(null)}
          >
            {t("viewGotIt")}
          </button>
        </div>
      )}
      {note && (
        <p className="shrink-0 px-4 py-1.5 text-[12.5px] text-urgent" role="alert">
          {note}
        </p>
      )}
      {reading && (shown.kind === "doc" || body.trim()) && (
        <Reader
          key={shown.kind === "doc" ? shown.paper.file : shown.path}
          body={body}
          title={title}
          under={under}
          stranger={shown.kind === "file"}
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
          onEdit={() => editIn(true)}
          onExport={(at) => setMenu({ ...at, only: "export" })}
          onFound={() => setFinding(false)}
          onDoc={(file) => void viewGo(file).catch(fail)}
          onOpen={
            shown.kind === "doc" ? (reference) => void opened(reference).catch(fail) : undefined
          }
        />
      )}
      {shown?.kind === "file" && !body.trim() && (
        <Problem
          trouble={{ kind: "empty" }}
          name={shown.name}
          place={shown.place}
          onRetry={look}
          onEdit={() => editIn(true)}
        />
      )}
      {shown?.kind === "missed" && (
        <Problem
          trouble={troubleOf(shown.refusal, shown.path)}
          name={shown.name}
          place={shown.place}
          onReveal={
            shown.path.startsWith(SHARED)
              ? undefined
              : () => void revealItemInDir(shown.path).catch(fail)
          }
          onBrowse={
            shown.path.startsWith(SHARED) ? () => void openUrl(shown.path).catch(fail) : undefined
          }
          onRetry={look}
        />
      )}
      {shown?.kind === "widget" && (
        <Stage lent={shown.lent} preset={preset} actual={actual} dark={dark} again={again} />
      )}
      {code && shown?.kind === "widget" && "source" in shown.lent && (
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
            {shown.lent.source}
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
