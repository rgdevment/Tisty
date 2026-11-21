import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { type Showing, viewShown } from "./core";
import { adopt, t } from "./locales";
import { pieceSave } from "./pieceWindow";
import { saidPlainly } from "./refusal";
import Diagramming from "./ui/drawing/Diagramming";
import Menu from "./ui/Menu";
import { modKey } from "./ui/macKeys";
import Quill from "./ui/Quill";
import { darkNow } from "./ui/theming";
import { type Acts, choicesFor, keysFor } from "./ui/viewer/acting";
import Header, { titled } from "./ui/viewer/Header";
import PieceCode from "./ui/viewer/PieceCode";
import PiecePreview from "./ui/viewer/PiecePreview";
import Stage, { type Preset } from "./ui/viewer/Stage";
import WindowChrome from "./ui/WindowChrome";

// A diagram's window is its own thing: none of the piece's state is set up for it.
const DRAWN_PAUSE = 250;
const SAVED_PAUSE = 700;

export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  // One look at a time: a second mount in development never fetches twice.
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
  return <Piece shown={shown} trouble={trouble} />;
}

function Piece({ shown, trouble }: { shown: Showing | null; trouble: string | null }) {
  const [note, setNote] = useState<string | null>(null);
  // Each answer from the window, good or bad, replaces what the last one said.
  useEffect(() => setNote(trouble), [trouble, shown]);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const [preset, setPreset] = useState<Preset>("fit");
  const [actual, setActual] = useState(false);
  const [dark, setDark] = useState<boolean | null>(null);
  const [again, setAgain] = useState(0);
  const [code, setCode] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [draft, setDraft] = useState<string | null>(null);
  const unsaved = useRef<string | null>(null);
  const pause = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const drawing = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const fail = (problem: unknown) => setNote(saidPlainly(problem));

  const opensOnCode = shown?.kind === "piece" && shown.code;
  useEffect(() => {
    if (opensOnCode) setCode(true);
  }, [opensOnCode]);

  // An empty piece leaves nothing to find again, so blank words wait for some before they are saved.
  const drafted = (now: string) => {
    setDraft(now);
    clearTimeout(drawing.current);
    drawing.current = setTimeout(() => setPreview(now), DRAWN_PAUSE);
    clearTimeout(pause.current);
    unsaved.current = now.trim() ? now : null;
    if (unsaved.current !== null) pause.current = setTimeout(() => void flush(), SAVED_PAUSE);
  };
  const flush = (): Promise<void> => {
    clearTimeout(pause.current);
    const now = unsaved.current;
    unsaved.current = null;
    if (now === null) return Promise.resolve();
    return pieceSave(now)
      .then(() => setNote(null))
      .catch(fail);
  };
  const flushing = useRef(flush);
  flushing.current = flush;
  useEffect(() => {
    const closing = getCurrentWindow().onCloseRequested(() => flushing.current());
    return () => {
      closing.then((off) => off()).catch(() => {});
    };
  }, []);

  const acts: Acts = {
    shown,
    dark,
    actual,
    fail,
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
  escaping.current = () => setCode(false);
  useEffect(() => {
    const heard = (e: KeyboardEvent) => {
      if (e.key === "Escape") return escaping.current();
      if (!modKey(e)) return;
      const act = keysFor(keys.current, e);
      if (!act) return;
      e.preventDefault();
      act();
    };
    window.addEventListener("keydown", heard);
    return () => window.removeEventListener("keydown", heard);
  }, []);

  const { title } = titled(shown);

  return (
    <main className="relative flex h-full flex-col bg-desk font-sans">
      <WindowChrome flush />
      <Header
        shown={shown}
        preset={preset}
        menu={Boolean(menu)}
        onPreset={setPreset}
        onAgain={acts.again}
        onMore={setMenu}
      />
      {note && (
        <p className="shrink-0 px-4 py-1.5 text-[12.5px] text-urgent" role="alert">
          {note}
        </p>
      )}
      {!trouble && !shown && <Quill said={t("viewOpening")} />}
      {shown?.kind === "piece" && (
        <div className="flex min-h-0 flex-1">
          {code && (
            <PieceCode
              source={draft ?? shown.source}
              writable={shown.writable}
              onDraft={drafted}
              onClose={() => setCode(false)}
            />
          )}
          <div className="flex min-h-0 min-w-0 flex-1 flex-col">
            {shown.held === "widget" ? (
              <Stage
                source={preview ?? shown.source}
                preset={preset}
                actual={actual}
                dark={dark}
                again={again}
              />
            ) : (
              <PiecePreview held={shown.held} source={preview ?? shown.source} label={title} />
            )}
          </div>
        </div>
      )}
      {menu && (
        <Menu
          at={menu}
          choices={choicesFor(acts)}
          label={t("viewMore")}
          onClose={() => setMenu(null)}
        />
      )}
    </main>
  );
}
