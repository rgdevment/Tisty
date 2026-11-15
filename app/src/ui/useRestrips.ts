import { useEffect, useRef, useState } from "react";
import { diaryClosed } from "../diary";

// The first look at a page is what it was drawn with; only a later round that tells otherwise draws it again.
export function useRestrips(file: string | undefined, body: string, round: number): number {
  const [restrips, setRestrips] = useState(0);
  const seen = useRef<{ file: string; said: string } | null>(null);
  const bodyNow = useRef(body);
  bodyNow.current = body;
  useEffect(() => {
    if (!file) {
      seen.current = null;
      return;
    }
    let live = true;
    diaryClosed(file, bodyNow.current)
      .then((said) => {
        if (!live) return;
        const was = seen.current;
        seen.current = { file, said };
        if (was?.file === file && was.said !== said) setRestrips((n) => n + 1);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [file, round]);
  return restrips;
}
