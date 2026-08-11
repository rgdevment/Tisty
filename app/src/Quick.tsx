import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { capture, shortcut, snapshot, type Snapshot } from "./core";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import CaptureField from "./ui/CaptureField";

const hint = (keys: string | null) =>
  keys ? `${fill("quickKeys", keys)} · ${t("quickHint")}` : t("quickHint");

/** Capture without the window: one field, and it is gone again. */
export default function Quick() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kept, setKept] = useState<string>();
  const [round, setRound] = useState(0);
  const going = useRef<ReturnType<typeof setTimeout>>(null);
  const [keys, setKeys] = useState<string | null>(null);

  useEffect(() => {
    const window = getCurrentWindow();
    const away = () => void window.hide();

    const look = () =>
      snapshot({})
        .then((fresh) => {
          adopt(fresh.locale);
          setData(fresh);
        })
        .catch((e) => setError(saidPlainly(e)));

    // Read again every time it appears: this window is hidden, never closed,
    // so its lists and tags would otherwise be as old as the last showing.
    const stop = window.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        away();
        return;
      }
      // Outlives the hiding: uncancelled, it takes the next capture down.
      if (going.current) clearTimeout(going.current);
      setKept(undefined);
      setError(null);
      setRound((n) => n + 1);
      void look();
    });

    void look();
    shortcut()
      .then(setKeys)
      .catch(() => {});

    const escape = (e: KeyboardEvent) => {
      if (e.key === "Escape") away();
    };
    globalThis.addEventListener("keydown", escape);

    return () => {
      void stop.then((off) => off());
      globalThis.removeEventListener("keydown", escape);
      if (going.current) clearTimeout(going.current);
    };
  }, []);

  // Frameless: an empty render is an invisible rectangle eating clicks.
  if (!data) {
    return (
      <div className="flex h-full flex-col justify-center rounded-xl border border-hair bg-bg px-4 py-3 shadow-2xl">
        <p className="px-1 text-[13.5px] text-soft">{error ?? t("addTask")}</p>
        <p className="px-1 pt-1 text-[11px] text-faint">{hint(keys)}</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col justify-center rounded-xl border border-hair bg-bg px-4 py-3 shadow-2xl">
      {kept ? (
        <p className="px-1 text-[13.5px] text-soft">
          <span className="text-accent">✓</span> {kept}
        </p>
      ) : (
        <CaptureField
          key={round}
          invite={t("addTask")}
          lists={data.lists}
          tags={data.tags}
          onCapture={(written, edits) => {
            setError(null);
            return capture(written, {}, edits).then((task) => {
              setKept(task.title);
              void emit("captured");
              going.current = setTimeout(() => void getCurrentWindow().hide(), 900);
              return task;
            });
          }}
          onError={setError}
        />
      )}
      {error && <p className="px-1 pt-1 text-xs text-urgent">{error}</p>}
      {!kept && <p className="px-1 pt-1 text-[11px] text-faint">{hint(keys)}</p>}
    </div>
  );
}
