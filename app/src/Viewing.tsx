import { emitTo } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { docRead, type Lent, type Showing, viewImport, viewShown } from "./core";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import { sure } from "./sure";
import Editor from "./ui/Editor";
import WindowChrome, { clearOfChrome, onMac } from "./ui/WindowChrome";
import { type Lending, lend, lendKept } from "./ui/widgeting";

const action =
  "shrink-0 rounded-[9px] border border-line px-2.5 py-[3px] text-[12px] text-soft hover:bg-hover hover:text-ink disabled:border-hair disabled:text-faint disabled:hover:bg-transparent";

function Widget({ lent }: { lent: Lent }) {
  const drawn = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const at = drawn.current;
    if (!at) return;
    const lending: Lending =
      "reference" in lent ? lendKept(at, lent.reference) : lend(at, lent.source);
    return () => lending.drop();
  }, [lent]);
  return <div ref={drawn} className="viewed-widget min-h-0 flex-1 bg-sheet" />;
}

export default function Viewing() {
  useState(() => adopt(new URLSearchParams(window.location.search).get("locale") ?? undefined));
  const [shown, setShown] = useState<Showing | null>(null);
  const [body, setBody] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [kept, setKept] = useState(false);

  useEffect(() => {
    viewShown()
      .then((one) => {
        setShown(one);
        if (one.kind === "file") setBody(one.body);
        if (one.kind === "doc") return docRead(one.file).then(setBody);
      })
      .catch((e) => setProblem(saidPlainly(e)));
  }, []);

  const keep = (name: string) =>
    sure(fill("viewKeepSure", name))
      .then((yes) => (yes ? viewImport().then(() => setKept(true)) : undefined))
      .catch((e) => setProblem(saidPlainly(e)));

  const title =
    shown?.kind === "file"
      ? shown.name
      : shown?.kind === "doc"
        ? shown.title || t("untitledDoc")
        : t("viewWidgetTitle");

  return (
    <main className="flex h-full flex-col bg-desk font-sans">
      <WindowChrome />
      <header
        data-tauri-drag-region
        className={`flex h-11 shrink-0 items-center gap-2 ${onMac ? "pl-20" : "pl-4"} ${clearOfChrome}`}
      >
        <div data-tauri-drag-region className="flex min-w-0 flex-1 items-baseline gap-2">
          <h1 data-tauri-drag-region className="truncate text-[13px] font-semibold text-ink">
            {title}
          </h1>
          {shown && shown.kind !== "widget" && (
            <span data-tauri-drag-region className="shrink-0 text-[11.5px] text-faint">
              {t("viewOnlyReading")}
            </span>
          )}
        </div>
        {shown?.kind === "file" && (
          <button
            type="button"
            className={action}
            disabled={kept}
            onClick={() => void keep(shown.name)}
          >
            {t(kept ? "viewKept" : "viewKeep")}
          </button>
        )}
      </header>
      {problem && <p className="px-6 pb-2 text-[12.5px] text-urgent">{problem}</p>}
      {shown?.kind === "widget" && <Widget lent={shown.lent} />}
      {shown && shown.kind !== "widget" && body !== null && (
        <div className="viewed-doc mx-auto flex min-h-0 w-full max-w-[900px] flex-1 flex-col px-4 pb-4">
          <Editor
            value={body}
            reading
            stranger={shown.kind === "file"}
            label={title}
            onWrite={() => {}}
            onDoc={(id) => void emitTo("main", "showdoc", id)}
          />
        </div>
      )}
    </main>
  );
}
