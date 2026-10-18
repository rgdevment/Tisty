import type { Chosen } from "./views";

interface Seen {
  chosen: Chosen;
  open: boolean;
  calm: boolean;
}

export const layoutOf = ({ chosen, open, calm }: Seen) => {
  const outside =
    chosen.named === "docs" ||
    chosen.named === "keeping" ||
    chosen.named === "aboutScreen" ||
    chosen.named === "help";
  const sheet = open && !outside;
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
  const lane = chosen.named !== "spread" && aside ? "@min-[884px]:pr-[324px]" : "";
  return { sheet, aside, quiet, papered, lane };
};
