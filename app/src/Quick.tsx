import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { jot } from "./brainstorm";
import { capture } from "./capturing";
import { type Priority, type Snapshot, shortcut, snapshot } from "./core";
import { diaryWrite } from "./diary";
import { every, turnOf } from "./examples";
import { adopt, fill, t } from "./locales";
import { IDEA_AT_MOST } from "./notes";
import { said } from "./quadrants";
import { saidPlainly } from "./refusal";
import CaptureField from "./ui/CaptureField";
import QuickField from "./ui/QuickField";
import { winksNow } from "./wink";

const invited = (): string => (winksNow() ? t("taskWink") : turnOf(every(), Math.random()));

type Mode = "task" | "idea" | "diary";

const MODES: Mode[] = ["task", "idea", "diary"];

const turned = (was: Mode, back: boolean): Mode =>
  MODES[(MODES.indexOf(was) + (back ? MODES.length - 1 : 1)) % MODES.length] ?? "task";

const NAMED = { task: "modeTask", idea: "modeIdea", diary: "modeDiary" } as const;

export default function Quick() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kept, setKept] = useState<string>();
  const [round, setRound] = useState(0);
  const [where, setWhere] = useState<Priority>("unset");
  const [mode, setMode] = useState<Mode>("task");
  const modeNow = useRef<Mode>("task");
  modeNow.current = mode;
  const going = useRef<ReturnType<typeof setTimeout>>(null);
  const [keys, setKeys] = useState<string | null>(null);
  useEffect(() => {
    if (round < 0) return;
    if (mode === "diary") return setKeys(null);
    shortcut(mode)
      .then(setKeys)
      .catch(() => setKeys(null));
  }, [mode, round]);
  const [example, setExample] = useState(() => t("addTask"));

  useEffect(() => {
    const window = getCurrentWindow();
    const away = () => void window.hide();

    const look = (opened = false) =>
      snapshot({ most: 0 })
        .then((fresh) => {
          adopt(fresh.locale);
          setData(fresh);
          if (opened) setExample(invited());
        })
        .catch((e) => setError(saidPlainly(e)));

    const stop = window.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        setWhere("unset");
        away();
        return;
      }
      if (going.current) clearTimeout(going.current);
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
      void look(true);
    });

    void look();

    const sown = listen<Priority | null>("sow", ({ payload }) => {
      setWhere(payload ?? "unset");
      setMode("task");
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
    });

    const opened = listen<{ door: string; again: boolean }>("door", ({ payload }) => {
      const asked: Mode = payload.door === "idea" ? "idea" : "task";
      if (payload.again && modeNow.current === asked) {
        away();
        return;
      }
      setMode(asked);
      setWhere("unset");
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
    });

    const leaving = (e: KeyboardEvent) => {
      if (e.key === "Escape") away();
      if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        setMode((was) => turned(was, e.shiftKey));
        setError(null);
      }
    };
    globalThis.addEventListener("keydown", leaving);

    return () => {
      void stop.then((off) => off());
      void sown.then((off) => off());
      void opened.then((off) => off());
      globalThis.removeEventListener("keydown", leaving);
      if (going.current) clearTimeout(going.current);
    };
  }, []);

  if (!data) {
    return (
      <Frame>
        <p className="text-[21px] leading-snug -tracking-[0.011em] text-soft">
          {error ?? t("addTask")}
        </p>
      </Frame>
    );
  }

  const done = (said: string) => {
    setKept(said);
    going.current = setTimeout(() => void getCurrentWindow().hide(), 900);
  };

  return (
    <Frame>
      {!kept && <Modes mode={mode} keys={keys} onSwitch={setMode} />}
      {kept ? (
        <p className="flex items-baseline gap-2.5 text-[21px] leading-snug -tracking-[0.011em]">
          <span className="text-[13px] text-accent">✓</span> {kept}
        </p>
      ) : mode !== "task" ? (
        <QuickField
          key={`${mode}-${round}`}
          icon={mode === "idea" ? "✎" : "❧"}
          hint={t(mode === "idea" ? "jotIdea" : "diaryWriting")}
          most={mode === "idea" ? IDEA_AT_MOST : undefined}
          onKeep={(text) => {
            setError(null);
            const kept = mode === "idea" ? jot(text, t("brainstorm")) : diaryWrite(text);
            return kept.then(() => {
              void emit("jotted");
              done(text.trim());
            });
          }}
          onError={setError}
        />
      ) : (
        <CaptureField
          key={round}
          invite={where === "unset" ? example : fill("addTo", said(where))}
          lists={data.lists}
          tags={data.tags}
          onCapture={(written, edits) => {
            setError(null);
            const asked = where === "unset" ? edits : { ...edits, priority: where };
            return capture(written, {}, asked).then((task) => {
              void emit("captured");
              done(task.title);
              return task;
            });
          }}
          onError={setError}
        />
      )}
      {error && <p className="pt-1 text-[11.5px] text-urgent">{error}</p>}
    </Frame>
  );
}

function Modes({
  mode,
  keys,
  onSwitch,
}: Readonly<{ mode: Mode; keys: string | null; onSwitch: (mode: Mode) => void }>) {
  return (
    <div className="mb-2 flex items-center gap-1 px-1 text-[11.5px]">
      {MODES.map((one) => (
        <button
          key={one}
          type="button"
          aria-pressed={one === mode}
          title={t("modeSwitch")}
          onClick={() => onSwitch(one)}
          className={`rounded-md px-2 py-px ${one === mode ? "bg-accent-soft text-accent" : "text-faint hover:text-soft"}`}
        >
          {t(NAMED[one])}
        </button>
      ))}
      {keys && (
        <kbd
          title={t(mode === "idea" ? "quickIdeaKeys" : "quickKeys")}
          className="ml-auto rounded-md bg-hover px-1.5 py-px font-sans text-[11.5px] text-soft"
        >
          {keys}
        </kbd>
      )}
    </div>
  );
}

function Frame({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="relative flex h-full flex-col justify-center overflow-hidden rounded-[10px] border border-hair bg-bg/70 px-[22px] py-5 shadow-lift-tall backdrop-blur-xl">
      {children}
    </div>
  );
}
