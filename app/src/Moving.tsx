import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { weigh } from "./format";
import { fill, t } from "./locales";

interface Along {
  done: number;
  whole: number;
}

export default function Moving() {
  const [along, setAlong] = useState<Along | null>(null);

  useEffect(() => {
    const off = listen<Along>("moving", ({ payload }) => setAlong(payload));
    return () => {
      void off.then((stop) => stop());
    };
  }, []);

  const share = along && along.whole > 0 ? Math.min(1, along.done / along.whole) : 0;

  return (
    <main data-tauri-drag-region className="flex h-screen flex-col justify-center gap-3 px-6">
      <h1 className="text-[13px] font-semibold">{t("movingTitle")}</h1>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("movingWhy")}</p>
      <div
        role="progressbar"
        aria-label={t("movingTitle")}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(share * 100)}
        className="h-1.5 overflow-hidden rounded-full bg-hover"
      >
        <div className="h-full bg-accent" style={{ width: `${share * 100}%` }} />
      </div>
      <p className="text-[11.5px] text-faint">
        {along ? fill("movingAlong", weigh(along.done), weigh(along.whole)) : t("movingStarting")}
      </p>
    </main>
  );
}
