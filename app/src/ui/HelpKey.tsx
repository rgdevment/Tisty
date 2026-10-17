import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { askedHere } from "../guide/asked";

export default function HelpKey({ onHelp }: { readonly onHelp: (topic?: string) => void }) {
  const now = useRef(onHelp);
  now.current = onHelp;
  useEffect(() => {
    const keyed = (event: KeyboardEvent) => {
      if (event.key !== "F1" || event.repeat) return;
      event.preventDefault();
      now.current(askedHere());
    };
    window.addEventListener("keydown", keyed);
    let drop: (() => void) | undefined;
    let gone = false;
    listen("help", () => now.current())
      .then((off) => {
        if (gone) off();
        else drop = off;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
      drop?.();
      window.removeEventListener("keydown", keyed);
    };
  }, []);
  return null;
}
