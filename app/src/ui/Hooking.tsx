import { useState } from "react";
import type { Wired } from "../core";
import { fill, t } from "../locales";

interface Props {
  one: Wired;
  held: boolean;
  onHook: (one: Wired, on: boolean) => void;
}

const button =
  "shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint hover:border-accent hover:text-accent";

export default function Hooking({ one, held, onHook }: Readonly<Props>) {
  const [asking, setAsking] = useState(false);
  const hooks = one.hooks;
  if (!hooks) return null;
  return (
    <div className="mt-2 rounded-[10px] border border-hair px-3 py-2">
      <div className="flex items-center gap-3">
        <span className="min-w-0 flex-1 text-[12.5px] text-soft">{t("hooksLabel")}</span>
        <span
          className={`shrink-0 rounded-full border px-2 py-0.5 text-[11.5px] whitespace-nowrap ${
            hooks.on ? "border-hue-green/40 text-hue-green" : "border-line text-soft"
          }`}
        >
          {t(hooks.on ? "hooksOn" : "hooksOff")}
        </span>
        <button
          type="button"
          disabled={held}
          aria-expanded={hooks.on ? undefined : asking}
          aria-label={`${t(hooks.on ? "hooksOut" : "hooksAdd")} · ${t("hooksLabel")}`}
          onClick={() => (hooks.on ? onHook(one, false) : setAsking(!asking))}
          className={`${button} ${hooks.on ? "border-line text-soft" : "border-accent text-accent"}`}
        >
          {t(hooks.on ? "hooksOut" : "hooksAdd")}
        </button>
      </div>
      {asking && !hooks.on && (
        <div className="mt-2 flex flex-col gap-2">
          <p className="text-[12.5px] leading-relaxed text-soft [overflow-wrap:anywhere]">
            {fill("hooksAsk", hooks.at)}
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              disabled={held}
              onClick={() => {
                setAsking(false);
                onHook(one, true);
              }}
              className="shrink-0 rounded-md bg-accent px-2.5 py-1 text-[12.5px] text-bg disabled:bg-hair disabled:text-soft"
            >
              {t("hooksYes")}
            </button>
            <button
              type="button"
              onClick={() => setAsking(false)}
              className={`${button} border-line text-soft`}
            >
              {t("hooksNo")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
