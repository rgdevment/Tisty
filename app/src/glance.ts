import { useEffect, useRef, useState } from "react";

// Long enough to have walked away from: past this, the notice waits rather than fading.
export const AT_A_GLANCE = 20_000;

export const SHOWN = 3_200;

export function useNote() {
  const [note, setNote] = useState<string | null>(null);
  const [waited, setWaited] = useState(false);
  const fading = useRef<number | undefined>(undefined);

  const showing = (text: string | null, fades: number | null) => {
    window.clearTimeout(fading.current);
    setNote(text);
    if (fades !== null) fading.current = window.setTimeout(() => setNote(null), fades);
  };

  const noted = (text: string, fades = SHOWN) => {
    setWaited(false);
    showing(text, fades);
  };

  const said = (text: string, since: number) => {
    const long = Date.now() - since >= AT_A_GLANCE;
    setWaited(long);
    showing(text, long ? null : SHOWN);
  };

  useEffect(() => {
    if (!waited) return;
    let ready = false;
    const soon = setTimeout(() => {
      ready = true;
    }, SHOWN);
    const gone = () => {
      if (!ready) return;
      window.clearTimeout(fading.current);
      setNote(null);
      setWaited(false);
    };
    window.addEventListener("pointerdown", gone);
    window.addEventListener("keydown", gone);
    return () => {
      clearTimeout(soon);
      window.removeEventListener("pointerdown", gone);
      window.removeEventListener("keydown", gone);
    };
  }, [waited, showing]);

  useEffect(() => () => window.clearTimeout(fading.current), []);

  return { note, noted, said };
}
