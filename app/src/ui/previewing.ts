import { useEffect } from "react";

// While a PDF preview fills the window, its own way back is the only control: closing from there would quit the app.
export const usePreviewing = (on: boolean) =>
  useEffect(() => {
    if (!on) return;
    const root = document.documentElement;
    root.dataset.previewing = "";
    return () => {
      delete root.dataset.previewing;
    };
  }, [on]);
