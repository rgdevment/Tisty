import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { placeRaised, placeRaisedTold } from "../core";
import { t } from "../locales";

// Said once, by the copy that raised the place: the others stop there until they update.
export default function Raised() {
  const [shown, setShown] = useState(false);

  useEffect(() => {
    const look = () =>
      void Promise.resolve()
        .then(placeRaised)
        .then((raised) => setShown(raised === true))
        .catch(() => {});
    look();
    window.addEventListener("focus", look);
    const brought = listen("brought", look);
    return () => {
      window.removeEventListener("focus", look);
      void brought.then((stop) => stop());
    };
  }, []);

  if (!shown) return null;

  const told = () => {
    setShown(false);
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
        <span className="block text-[12.5px] font-semibold text-ink">{t("placeRaised")}</span>
        {t("placeRaisedWhy")}
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
