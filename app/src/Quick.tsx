import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { capture, type Snapshot, snapshot } from "./core";
import { adopt, t } from "./locales";
import { saidPlainly } from "./refusal";
import CaptureField from "./ui/CaptureField";

export default function Quick() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kept, setKept] = useState<string>();
  const [round, setRound] = useState(0);
  const going = useRef<ReturnType<typeof setTimeout>>(null);

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

    const stop = window.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
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

  if (!data) {
    return (
      <Frame>
        <p className="text-[17px] leading-snug -tracking-[0.011em] text-soft">
          {error ?? t("addTask")}
        </p>
      </Frame>
    );
  }

  return (
    <Frame>
      {kept ? (
        <p className="flex items-baseline gap-2.5 text-[17px] leading-snug -tracking-[0.011em]">
          <span className="text-[13px] text-accent">✓</span> {kept}
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
      {error && <p className="pt-1 text-xs text-urgent">{error}</p>}
    </Frame>
  );
}

function Frame({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-full flex-col justify-center rounded-xl border border-hair bg-bg/85 px-[22px] py-5 shadow-lift-tall backdrop-blur-xl">
      {children}
    </div>
  );
}
