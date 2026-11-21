import { useCallback, useRef } from "react";
import { keepSettings, type Settings } from "../core";
import { t } from "../locales";
import type { Run, Which, Word } from "./Card";

interface Hands {
  setKept: (kept: Settings) => void;
  quietly: Run;
  tell: (word?: Word) => void;
  reread: () => void;
  onChanged: () => void;
}

// Each write carries the whole settings, so they go one after another and only the last answer is shown.
export function useRemember({ setKept, quietly, tell, reread, onChanged }: Hands) {
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const last = useRef(0);
  return useCallback(
    (next: Settings, card: Which) => {
      setKept(next);
      const ticket = ++last.current;
      const write = queue.current.then(() => keepSettings(next));
      queue.current = write.catch(() => {
        if (ticket === last.current) reread();
      });
      quietly(card, write, (now) => {
        if (ticket === last.current) setKept(now);
        tell({ card, text: t("settingsKept") });
        onChanged();
      });
    },
    [setKept, quietly, tell, reread, onChanged],
  );
}
