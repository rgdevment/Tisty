import { useEffect, useState } from "react";
import { fell } from "../Standing";

export type LoadedSheet = typeof import("./Sheet").default;

export const useSheet = (wanted: boolean): LoadedSheet | null => {
  const [sheet, setSheet] = useState<LoadedSheet | null>(null);
  useEffect(() => {
    if (!wanted) return;
    let live = true;
    void import("./Sheet").catch(fell).then(({ default: ready }) => {
      if (live) setSheet(() => ready);
    });
    return () => {
      live = false;
    };
  }, [wanted]);
  return sheet;
};
