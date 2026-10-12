import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { type Carrying, syncState } from "../core";
import { fill, t } from "../locales";

const leaf = (at: string) => at.split(/[\\/]/).filter(Boolean).pop() ?? at;

export const modeSaid = (state: Carrying): string => {
  if (state.mode === "later") return t("modeLater");
  if (state.mode === "folder" && state.chosen) return fill("modeFolder", leaf(state.chosen));
  return t("modeAlone");
};

export default function Mode({ onOpen }: { onOpen: () => void }) {
  const [state, setState] = useState<Carrying | null>(null);

  useEffect(() => {
    const look = () =>
      void syncState()
        .then(setState)
        .catch(() => {});
    look();
    window.addEventListener("focus", look);
    const changed = listen("unstuck", look);
    return () => {
      window.removeEventListener("focus", look);
      void changed.then((stop) => stop());
    };
  }, []);

  if (!state?.asked) return null;
  const said = modeSaid(state);
  return (
    <button
      type="button"
      onClick={onOpen}
      title={`${t("modeOpen")} · ${said}`}
      className="mx-1.5 min-w-0 flex-1 truncate rounded-md px-2 py-1 text-left text-[11.5px] text-faint hover:bg-hover hover:text-ink"
    >
      {said}
    </button>
  );
}
