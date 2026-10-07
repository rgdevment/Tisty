import type { Chosen } from "./views";

interface Seen {
  chosen: Chosen;
  open: boolean;
  mode: "columns" | "sheet";
  tight: boolean;
  calm: boolean;
}

export const layoutOf = ({ chosen, open, mode, tight, calm }: Seen) => {
  const outside =
    chosen.named === "docs" || chosen.named === "keeping" || chosen.named === "aboutScreen";
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
  return { sheet, beside, aside, quiet, papered, lane };
};
