import type { Chosen } from "./views";

export const DOC_ASIDE_FROM = 1082;
const PULSE_FROM = 884;
const TASK_BESIDE_FROM = 964;
const BOTH_FROM = 1536;

interface Seen {
  chosen: Chosen;
  open: boolean;
  mode: "columns" | "sheet";
  tight: boolean;
  calm: boolean;
}

export const layoutOf = ({ chosen, open, mode, tight, calm }: Seen) => {
  const outside = chosen.named === "docs" || chosen.named === "keeping" || chosen.named === "help";
  const sheet = open && !outside && (mode === "sheet" || tight);
  const beside = open && !outside && !sheet;
  const aside =
    (chosen.named === "tasks" || chosen.named === "tags" || chosen.list !== undefined) && !sheet;
  const quiet = calm && !open;
  const papered =
    chosen.named === "tasks" ||
    chosen.named === "tags" ||
    chosen.named === "archive" ||
    chosen.named === "lists" ||
    chosen.named === "spread" ||
    chosen.list !== undefined ||
    sheet;
  const lane =
    chosen.named === "spread"
      ? ""
      : beside
        ? aside
          ? "@min-[964px]:pr-[404px] @min-[1536px]:pr-[716px]"
          : "@min-[964px]:pr-[404px]"
        : aside
          ? "@min-[884px]:pr-[324px]"
          : "";
  const room =
    chosen.named === "docs"
      ? chosen.doc
        ? DOC_ASIDE_FROM
        : 0
      : beside && aside
        ? BOTH_FROM
        : aside
          ? PULSE_FROM
          : beside
            ? TASK_BESIDE_FROM
            : 0;
  return { sheet, beside, aside, quiet, papered, lane, room };
};
