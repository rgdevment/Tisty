import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useRef, useState } from "react";
import { jot } from "./brainstorm";
import { capture, type Priority, type Snapshot, shortcut, snapshot } from "./core";
import { adopt, fill, t } from "./locales";
import { said } from "./quadrants";
import { saidPlainly } from "./refusal";
import CaptureField from "./ui/CaptureField";
import IdeaField from "./ui/IdeaField";

const anyOf = (all: string[]): string => all[Math.floor(Math.random() * all.length)] ?? "";

const WINKS_ONCE_IN = 100;

const invited = (): string =>
  Math.random() * WINKS_ONCE_IN < 1 ? t("taskWink") : anyOf(t("taskExamples").split("\n"));

export default function Quick() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kept, setKept] = useState<string>();
  const [round, setRound] = useState(0);
  const [where, setWhere] = useState<Priority>("unset");
  const [jotting, setJotting] = useState(false);
  const jottingNow = useRef(false);
  jottingNow.current = jotting;
  const going = useRef<ReturnType<typeof setTimeout>>(null);
  const [keys, setKeys] = useState<string | null>(null);
  useEffect(() => {
    if (round < 0) return;
    shortcut(jotting ? "idea" : "task")
      .then(setKeys)
      .catch(() => setKeys(null));
  }, [jotting, round]);
  const locale = data?.locale;
  const example = useMemo(
    () => (round >= 0 && locale !== undefined ? invited() : t("addTask")),
    [round, locale],
  );

  useEffect(() => {
    const window = getCurrentWindow();
    const away = () => void window.hide();

    const look = () =>
      snapshot({ most: 0 })
        .then((fresh) => {
          adopt(fresh.locale);
          setData(fresh);
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
      void look();
    });

    void look();

    const sown = listen<Priority | null>("sow", ({ payload }) => {
      setWhere(payload ?? "unset");
      setJotting(false);
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
    });

    const opened = listen<{ door: string; again: boolean }>("door", ({ payload }) => {
      const idea = payload.door === "idea";
      if (payload.again && jottingNow.current === idea) {
        away();
        return;
      }
      setJotting(idea);
      setWhere("unset");
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
    });

    const leaving = (e: KeyboardEvent) => {
      if (e.key === "Escape") away();
      if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        setJotting((was) => !was);
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
      {!kept && <Modes jotting={jotting} keys={keys} onSwitch={setJotting} />}
      {kept ? (
        <p className="flex items-baseline gap-2.5 text-[21px] leading-snug -tracking-[0.011em]">
          <span className="text-[13px] text-accent">✓</span> {kept}
        </p>
      ) : jotting ? (
        <IdeaField
          key={round}
          onJot={(idea) => {
            setError(null);
            return jot(idea, t("brainstorm")).then(() => {
              void emit("jotted");
              done(idea.trim());
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
  jotting,
  keys,
  onSwitch,
}: Readonly<{ jotting: boolean; keys: string | null; onSwitch: (jotting: boolean) => void }>) {
  return (
    <div className="mb-2 flex items-center gap-1 px-1 text-[11.5px]">
      {[false, true].map((idea) => (
        <button
          key={String(idea)}
          type="button"
          aria-pressed={idea === jotting}
          title={t("modeSwitch")}
          onClick={() => onSwitch(idea)}
          className={`rounded-md px-2 py-px ${idea === jotting ? "bg-accent-soft text-accent" : "text-faint hover:text-soft"}`}
        >
          {t(idea ? "modeIdea" : "modeTask")}
        </button>
      ))}
      {keys && (
        <kbd
          title={t(jotting ? "quickIdeaKeys" : "quickKeys")}
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
