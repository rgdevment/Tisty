import { ask } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import type { LogEntry } from "../core";
import { wroteAt } from "../format";
import { t } from "../locales";
import { signedBy } from "../who";
import Prose from "./Prose";

interface Props {
  entries: LogEntry[];
  steps?: string[];
  onError?: (problem: unknown) => void;
  onWhole?: () => void;
  onDoc?: (id: string) => void;
  onWrite: (body: string, entry?: string) => void;
}

export default function Journal({ entries, steps, onError, onWhole, onDoc, onWrite }: Props) {
  const [draft, setDraft] = useState(0);

  return (
    <>
      <Prose
        key={draft}
        value=""
        hint={t("writeLog")}
        label={t("journal")}
        steps={steps}
        onError={onError}
        onDoc={onDoc}
        rows={1}
        catches
        onWrite={(body) => {
          if (body.trim()) onWrite(body);
          setDraft((n) => n + 1);
        }}
      />

      {entries
        .filter((entry) => entry.body.trim())
        .map((entry) => (
          <div key={entry.id} className="group border-t border-hair py-2.5">
            <div className="mb-1 flex items-baseline gap-1.5 px-1.5 text-[11.5px] text-faint">
              <time className="flex items-baseline gap-1.5">
                {wroteAt(entry.at, entry.tz)}
                {signedBy(entry.by, entry.via) && (
                  <span className="text-hue-teal">{signedBy(entry.by, entry.via)}</span>
                )}
              </time>
              <button
                type="button"
                aria-label={`${t("remove")} ${wroteAt(entry.at, entry.tz)}`}
                onClick={() =>
                  ask(t("journalEraseSure"), { kind: "warning" })
                    .then((sure) => sure && onWrite("", entry.id))
                    .catch(onError)
                }
                className="ml-auto flex h-4 w-4 shrink-0 items-center justify-center rounded-md text-faint opacity-0 outline-none group-hover:opacity-100 hover:bg-line hover:text-ink focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-accent"
              >
                ×
              </button>
            </div>
            <Prose
              value={entry.body}
              hint={t("writeLog")}
              label={entry.body}
              steps={steps}
              onError={onError}
              onWhole={onWhole}
              onDoc={onDoc}
              rows={1}
              onWrite={(body) => body.trim() && onWrite(body, entry.id)}
            />
          </div>
        ))}
    </>
  );
}
