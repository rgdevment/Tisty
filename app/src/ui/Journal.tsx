import { useState } from "react";
import type { LogEntry } from "../core";
import { wroteAt } from "../format";
import { t } from "../locales";
import Prose from "./Prose";

interface Props {
  entries: LogEntry[];
  known: string[];
  onError?: (problem: unknown) => void;
  onWhole?: () => void;
  onWrite: (body: string, entry?: string) => void;
}

export default function Journal({ entries, known, onError, onWhole, onWrite }: Props) {
  const [draft, setDraft] = useState(0);

  return (
    <>
      <Prose
        key={draft}
        value=""
        hint={t("writeLog")}
        label={t("journal")}
        known={known}
        onError={onError}
        rows={1}
        catches
        onWrite={(body) => {
          if (body.trim()) onWrite(body);
          setDraft((n) => n + 1);
        }}
      />

      {entries.map((entry) => (
        <div key={entry.id} className="border-t border-hair py-2.5">
          <time className="mb-1 block px-1.5 text-[11.5px] text-faint">
            {wroteAt(entry.at, entry.tz)}
          </time>
          <Prose
            value={entry.body}
            hint={t("writeLog")}
            label={entry.body}
            known={known}
            onError={onError}
            onWhole={onWhole}
            rows={1}
            onWrite={(body) => body.trim() && onWrite(body, entry.id)}
          />
        </div>
      ))}
    </>
  );
}
