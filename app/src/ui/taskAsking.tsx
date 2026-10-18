import { open as pick } from "@tauri-apps/plugin-dialog";
import { type ReactNode, useMemo, useRef, useState } from "react";
import { attach, type Filed } from "../core";
import { keptAll } from "../kept";
import { t } from "../locales";
import Picking, { type Picked } from "./Picking";
import type { Asking } from "./slashing";

const listed = (chosen: string | string[] | null): string[] => {
  if (Array.isArray(chosen)) return chosen;
  return chosen ? [chosen] : [];
};

// A task's page offers what a document's does, save its own pages and a new document.
export function useTaskAsks(
  papers: readonly Filed[] | undefined,
  onError?: (problem: unknown) => void,
): { asking?: Asking; picker: ReactNode } {
  const [picked, setPicked] = useState<Picked | null>(null);
  const failed = useRef(onError);
  failed.current = onError;
  const asking = useMemo<Asking | undefined>(
    () =>
      papers && {
        kinds: ["doc", "icon", "file"],
        ask: (kind, at, put) => {
          const said = (text: string) => failed.current?.(text);
          if (kind !== "file") return setPicked({ kind, at, put });
          pick({ multiple: true })
            .then((chosen) =>
              keptAll(
                listed(chosen).map((one) => attach(one, undefined, true)),
                said,
              ),
            )
            .then((text) => {
              if (text && !put(text)) said(t("attachmentLost"));
            })
            .catch((e) => failed.current?.(e));
        },
      },
    [papers],
  );
  const picker = picked && papers && (
    <Picking
      picked={picked}
      pages={[]}
      others={papers.filter((one) => !one.away)}
      onMade={() => {}}
      onLost={() => {}}
      onError={(problem) => failed.current?.(problem)}
      onDone={() => setPicked(null)}
    />
  );
  return { asking, picker };
}
