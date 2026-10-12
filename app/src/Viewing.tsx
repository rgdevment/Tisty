import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { type Showing, viewImport, viewShown } from "./core";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import { sure } from "./sure";
import Editor from "./ui/Editor";
import { type Lending, lend, lendKept } from "./ui/widgeting";
import { disarmed } from "./viewed";

const toggled = async (leaving = false) => {
  const window = getCurrentWindow();
  const full = await window.isFullscreen();
  if (leaving && !full) return;
  await window.setFullscreen(!full);
};

function Widget({ shown }: { shown: Extract<Showing, { kind: "widget" }> }) {
  const drawn = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const at = drawn.current;
    if (!at) return;
    let lending: Lending | null = null;
    if (shown.reference) lending = lendKept(at, shown.reference);
    else if (shown.source) lending = lend(at, shown.source);
    return () => lending?.drop();
  }, [shown]);
  return <div ref={drawn} className="viewed-widget min-h-0 flex-1" />;
}

export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [kept, setKept] = useState(false);

  useEffect(() => {
    viewShown()
      .then(setShown)
      .catch((e) => setProblem(saidPlainly(e)));
    const keys = (e: KeyboardEvent) => {
      if (e.key === "F11") {
        e.preventDefault();
        void toggled();
      }
      if (e.key === "Escape") void toggled(true);
    };
    window.addEventListener("keydown", keys);
    return () => window.removeEventListener("keydown", keys);
  }, []);

  const keep = (name: string) =>
    sure(fill("viewKeepSure", name))
      .then((yes) => (yes ? viewImport().then(() => setKept(true)) : undefined))
      .catch((e) => setProblem(saidPlainly(e)));

  return (
    <main className="flex h-screen flex-col bg-sheet">
      <header className="flex items-center gap-2 border-b border-line px-4 py-2">
        <h1 className="min-w-0 flex-1 truncate text-[13px] font-semibold">
          {shown?.kind === "file" ? shown.name : t("viewWidgetTitle")}
        </h1>
        {shown?.kind === "file" && (
          <button
            type="button"
            className="rounded px-2 py-1 text-[12px] hover:bg-hover disabled:opacity-50"
            disabled={kept}
            onClick={() => void keep(shown.name)}
          >
            {t(kept ? "viewKept" : "viewKeep")}
          </button>
        )}
        <button
          type="button"
          className="rounded px-2 py-1 text-[12px] hover:bg-hover"
          title="F11"
          onClick={() => void toggled()}
        >
          {t("viewFull")}
        </button>
      </header>
      {problem && <p className="px-4 py-2 text-[12.5px] text-urgent">{problem}</p>}
      {shown?.kind === "file" && (
        <div className="flex min-h-0 flex-1 flex-col overflow-auto px-6 py-4">
          <Editor value={disarmed(shown.body)} reading label={shown.name} onWrite={() => {}} />
        </div>
      )}
      {shown?.kind === "widget" && <Widget shown={shown} />}
    </main>
  );
}
