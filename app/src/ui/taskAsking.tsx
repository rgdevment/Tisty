import { open as pick } from "@tauri-apps/plugin-dialog";
import { type ReactNode, useMemo, useRef, useState } from "react";
import type { Asking } from "tisty-editor/edit";
import type { Filed } from "../core";
import { keptAll } from "../kept";
import { t } from "../locales";
import { attachedPicked } from "../reshaping";
import Picking, { type Picked } from "./Picking";

// A task's page offers what a document's does, save its own pages and a new document.
export function useTaskAsks(
  papers: readonly Filed[] | undefined,
  onError?: (problem: unknown) => void,
  whole?: string,
): { asking?: Asking; picker: ReactNode } {
  const [picked, setPicked] = useState<Picked | null>(null);
  const failed = useRef(onError);
  failed.current = onError;
  const asking = useMemo<Asking | undefined>(
    () =>
      papers && {
        kinds: whole ? ["doc", "newTask", "task", "icon", "file"] : ["doc", "task", "icon", "file"],
        ask: (kind, at, put) => {
          const said = (text: string) => failed.current?.(text);
          if (kind !== "file") return setPicked({ kind, at, put, whole });
          pick({ multiple: true })
            .then(attachedPicked)
            .then((all) => keptAll(all, said))
            .then((text) => {
              if (text && !put(text)) said(t("attachmentLost"));
            })
            .catch((e) => failed.current?.(e));
        },
      },
    [papers, whole],
  );
  const picker = picked && papers && (
    <Picking
      picked={picked}
      pages={[]}
      others={papers.filter((one) => !one.away)}
      onMade={() => {}}
      onLost={() => failed.current?.(t("attachmentLost"))}
      onError={(problem) => failed.current?.(problem)}
      onDone={() => setPicked(null)}
    />
  );
  return { asking, picker };
}
