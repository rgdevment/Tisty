import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { placeRaised, placeRaisedTold } from "../core";
import { fill, t } from "../locales";

// Said once, by the copy that raised the place: the others stop there until they update.
export default function Raised() {
  const [needs, setNeeds] = useState<string | null>(null);

  useEffect(() => {
    const look = () =>
      void Promise.resolve()
        .then(placeRaised)
        .then((raised) => setNeeds(typeof raised === "string" ? raised : null))
        .catch(() => {});
    look();
    window.addEventListener("focus", look);
    const brought = listen("brought", look);
    return () => {
      window.removeEventListener("focus", look);
      void brought.then((stop) => stop());
    };
  }, []);

  if (!needs) return null;

  const told = () => {
    setNeeds(null);
    void Promise.resolve()
      .then(placeRaisedTold)
      .catch(() => {});
  };

  return (
    <div
      role="status"
      className="shadow-lift fixed right-4 bottom-5 z-[60] flex w-[320px] items-start gap-2.5 rounded-[10px] border border-hue-amber/40 bg-bg px-3.5 py-2 text-[11.5px] leading-relaxed text-soft"
    >
      <span className="min-w-0 flex-1">
        <span className="block text-[12.5px] font-semibold text-ink">
          {fill("placeRaised", needs)}
        </span>
        {fill("placeRaisedWhy", needs)}
      </span>
      <button
        type="button"
        onClick={told}
        className="shrink-0 rounded-md border border-hair px-1.5 py-0.5 text-ink hover:bg-hover"
      >
        {t("placeRaisedTold")}
      </button>
    </div>
  );
}
