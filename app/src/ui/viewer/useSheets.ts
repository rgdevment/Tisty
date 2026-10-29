import { useState } from "react";
import { laidIn } from "../../front";

type Chosen = {
  readonly which: string | null;
  readonly sheets?: boolean;
  readonly landscape?: boolean;
};

// A choice belongs to the file it was made on, so the next file opens as it asks from its first frame.
export const useSheets = (which: string | null, body: string) => {
  const [chosen, setChosen] = useState<Chosen>({ which });
  const laid = laidIn(body);
  const mine: Chosen = chosen.which === which ? chosen : { which };
  if (mine !== chosen) setChosen(mine);
  const sheets = mine.sheets ?? laid.sheets;
  const landscape = sheets && (mine.landscape ?? laid.landscape);
  return {
    sheets,
    landscape,
    onSheets: (on: boolean) => setChosen({ ...mine, sheets: on }),
    onLandscape: (across: boolean) => setChosen({ ...mine, sheets: true, landscape: across }),
  };
};
