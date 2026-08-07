import { useEffect, useState } from "react";
import { read, type Parsed, type Task } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";

interface Props {
  onCapture: (text: string) => Promise<Task>;
  onError: (message: string) => void;
}

const SETTLES = 120;

export default function CaptureField({ onCapture, onError }: Props) {
  const [text, setText] = useState("");
  const [seen, setSeen] = useState<Parsed | null>(null);

  useEffect(() => {
    if (!text.trim()) {
      setSeen(null);
      return;
    }
    const timer = setTimeout(() => {
      read(text)
        .then(setSeen)
        .catch(() => setSeen(null));
    }, SETTLES);
    return () => clearTimeout(timer);
  }, [text]);

  return (
    <div className="w-full">
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onCapture(text)
            .then(() => setText(""))
            .catch((problem) => onError(String(problem)));
        }}
      >
        <input
          autoFocus
          value={text}
          onChange={(e) => setText(e.target.value)}
          aria-label={t("capture")}
          className="w-full rounded-[9px] border border-line bg-bg px-3.5 py-2.5 text-sm outline-none focus:border-accent focus:ring-[3px] focus:ring-accent-soft"
        />
      </form>

      <div className="mt-2 flex gap-2.5 overflow-hidden px-1 text-[11.5px] whitespace-nowrap text-faint">
        {seen ? <Reading seen={seen} /> : <Hint />}
      </div>
    </div>
  );
}

function Hint() {
  return (
    <>
      <span>
        <Key>#</Key> {t("fieldList")}
      </span>
      <span>
        <Key>@</Key> {t("fieldTag")}
      </span>
      <span>
        <Key>!</Key> {t("fieldPriority")}
      </span>
      <span>{t("hintDates")}</span>
      <span>
        <Key>/</Key> {t("hintPick")}
      </span>
    </>
  );
}

function Reading({ seen }: { seen: Parsed }) {
  const said: [string, string][] = [];

  if (seen.date) said.push([t("fieldDate"), whenLabel(seen.date)]);
  if (seen.deadline) said.push([t("fieldDeadline"), whenLabel(seen.deadline)]);
  if (seen.list) said.push([t("fieldList"), seen.list]);
  if (seen.tags.length) said.push([t("fieldTag"), seen.tags.join(" ")]);
  if (seen.priority && seen.priority < 4) {
    said.push([
      t("fieldPriority"),
      t(seen.priority === 1 ? "urgent" : seen.priority === 2 ? "high" : "medium"),
    ]);
  }

  if (said.length === 0) return <Hint />;

  return (
    <>
      {said.map(([label, value], i) => (
        <span key={label}>
          {i > 0 && <span className="mr-2.5">·</span>}
          <b className="font-medium text-soft">{label}</b> {value}
        </span>
      ))}
    </>
  );
}

function Key({ children }: { children: string }) {
  return <code className="rounded bg-hover px-1.5 py-px text-[11px] text-soft">{children}</code>;
}
