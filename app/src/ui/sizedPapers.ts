import { PAPER } from "tisty-editor/read";
import type { Paper } from "../core";
import { stored } from "../stored";

export const sizedPapers = stored<Record<string, Paper>>(
  "tisty.paper",
  (said) =>
    Object.fromEntries(
      Object.entries(said).filter(([, leaf]) => typeof leaf === "string" && leaf in PAPER),
    ) as Record<string, Paper>,
);
