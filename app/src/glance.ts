import { useEffect, useState } from "react";

// Long enough to have walked away from: past this, the notice waits rather than fading.
export const AT_A_GLANCE = 20_000;

export const SHOWN = 3_200;

export function useNote() {
  const [note, setNote] = useState<string | null>(null);
  const [waited, setWaited] = useState(false);

  const said = (text: string, since: number) => {
    const long = Date.now() - since >= AT_A_GLANCE;
    setNote(text);
    setWaited(long);
    if (!long) setTimeout(() => setNote(null), SHOWN);
  };

  useEffect(() => {
    if (!waited) return;
    let ready = false;
    const soon = setTimeout(() => {
      ready = true;
    }, SHOWN);
    const gone = () => {
      if (!ready) return;
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
  }, [waited]);

  return { note, setNote, said };
}
