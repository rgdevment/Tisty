import { useEffect, useRef, useState } from "react";
import type { LogEntry } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";

interface Props {
  entries: LogEntry[];
  onWrite: (body: string, entry?: string) => void;
}

export default function Journal({ entries, onWrite }: Props) {
  const [text, setText] = useState("");

  return (
    <>
      <textarea
        rows={1}
        value={text}
        placeholder={t("writeLog")}
        aria-label={t("journal")}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            if (text.trim()) {
              onWrite(text);
              setText("");
            }
          }
          if (e.key === "Escape") setText("");
        }}
        className="field-sizing-content mb-1 w-full resize-none rounded-md bg-transparent px-1.5 py-1 text-[13.5px] leading-relaxed outline-none placeholder:text-faint hover:bg-hover focus:bg-hover"
      />

      {entries.map((entry) => (
        <Entry key={entry.id} entry={entry} onWrite={onWrite} />
      ))}
    </>
  );
}

function Entry({ entry, onWrite }: { entry: LogEntry; onWrite: Props["onWrite"] }) {
  const [text, setText] = useState(entry.body);
  const dropped = useRef(false);
  useEffect(() => setText(entry.body), [entry.id, entry.body]);

  return (
    <div className="border-t border-hair py-2.5">
      <time className="mb-1 block px-1.5 text-[11.5px] text-faint">
        {whenLabel({ at: entry.at, tz: entry.tz ?? "", floating: false, has_time: true })}
      </time>
      <textarea
        rows={1}
        value={text}
        aria-label={entry.body}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => {
          if (dropped.current) {
            dropped.current = false;
            setText(entry.body);
            return;
          }
          const kept = text.trim();
          if (kept && kept !== entry.body) onWrite(kept, entry.id);
          else setText(entry.body);
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            dropped.current = true;
            e.currentTarget.blur();
          }
        }}
        className="field-sizing-content w-full resize-none rounded-md bg-transparent px-1.5 py-1 text-[13.5px] leading-relaxed outline-none hover:bg-hover focus:bg-hover"
      />
    </div>
  );
}
