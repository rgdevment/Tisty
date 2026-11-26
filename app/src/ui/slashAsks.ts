import { open as pick } from "@tauri-apps/plugin-dialog";
import { useMemo } from "react";
import type { Asked, Asking } from "tisty-editor/edit";
import { attachPasted } from "../core";
import { allowedKinds, folderAllows, type Lodged } from "../folderAllows";
import { keptAll, PASTED_UP_TO } from "../kept";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { attachedPicked } from "../reshaping";
import type { Picked } from "./Picking";

type Asks = {
  readonly bolted: boolean;
  /** A document of its own, not a page of another, which alone takes new pages. */
  readonly leafy: boolean;
  readonly lodged: Lodged;
  readonly file: string | undefined;
  readonly tags: readonly string[];
  readonly linkable: Asking["linkable"];
  readonly headsOf: Asking["headsOf"];
  readonly onError: (problem: unknown) => void;
  readonly onPicked: (picked: Picked) => void;
};

const LEAFY: readonly Asked[] = [
  "newPage",
  "page",
  "doc",
  "newDoc",
  "newTask",
  "task",
  "icon",
  "file",
  "diagram",
  "diagramFrom",
];

const PAGED: readonly Asked[] = [
  "doc",
  "newDoc",
  "newTask",
  "task",
  "icon",
  "file",
  "diagram",
  "diagramFrom",
];

/** What a document's `/` may ask for, as its folder allows it, and how each ask is answered. */
export const useSlashAsks = ({
  bolted,
  leafy,
  lodged,
  file,
  tags,
  linkable,
  headsOf,
  onError,
  onPicked,
}: Asks): Asking | undefined => {
  const { agents, diary, agentFile } = lodged;
  return useMemo<Asking | undefined>(
    () =>
      bolted
        ? undefined
        : {
            kinds: allowedKinds(leafy ? LEAFY : PAGED, folderAllows({ agents, diary, agentFile })),
            tags,
            linkable,
            headsOf,
            pasted: (files) =>
              keptAll(
                files.map((one) =>
                  one.size > PASTED_UP_TO
                    ? Promise.reject({ code: "attachmentTooBig", name: "650 MB" })
                    : one
                        .arrayBuffer()
                        .then((bytes) => attachPasted(one.name, one.type, new Uint8Array(bytes))),
                ),
                onError,
              ),
            lost: (notice) => onError(notice ?? t("attachmentLost")),
            ask: (kind, at, put, first, before) => {
              if (kind === "file") {
                // An attachment as a document's first words would become its title.
                if (first) return onError(t("fileNeedsTitle"));
                pick({ multiple: true })
                  .then(attachedPicked)
                  .then((all) => keptAll(all, onError))
                  .then((text) => {
                    if (text && !put(text)) onError(t("attachmentLost"));
                  })
                  .catch((error_) => onError(saidPlainly(error_)));
                return;
              }
              // A card as a document's first words would become its title, which the core refuses.
              if (kind !== "icon" && first)
                return onError(
                  t(
                    kind.startsWith("diagram")
                      ? "diagramNeedsTitle"
                      : kind === "task" || kind === "newTask"
                        ? "taskNeedsTitle"
                        : "leafNeedsTitle",
                  ),
                );
              onPicked({ kind, at, put, file, before });
            },
          },
    [bolted, leafy, agents, diary, agentFile, onError, onPicked, file, tags, linkable, headsOf],
  );
};
