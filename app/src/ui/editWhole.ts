import { useEffect, useState } from "react";
import { settings } from "../core";

export const WHOLE_TOLD = "edit-whole-told";

export const tellWhole = (whole: boolean) =>
  window.dispatchEvent(new CustomEvent(WHOLE_TOLD, { detail: whole }));

// Off unless the person chose it; a failed read keeps the block they know.
export function useEditWhole(): boolean {
  const [whole, setWhole] = useState(false);
  useEffect(() => {
    let live = true;
    const look = () => {
      settings()
        .then((kept) => {
          if (live) setWhole(Boolean(kept.editWhole));
        })
        .catch(() => {});
    };
    const told = (event: Event) => setWhole((event as CustomEvent<boolean>).detail);
    look();
    window.addEventListener("focus", look);
    window.addEventListener(WHOLE_TOLD, told);
    return () => {
      live = false;
      window.removeEventListener("focus", look);
      window.removeEventListener(WHOLE_TOLD, told);
    };
  }, []);
  return whole;
}
