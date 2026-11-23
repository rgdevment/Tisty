import { useState } from "react";
import { hooksAmiss } from "../agentsAmiss";
import type { Wired } from "../core";
import { fill, t, type Word } from "../locales";

interface Props {
  one: Wired;
  held: boolean;
  onHook: (one: Wired, on: boolean) => void;
  onFix?: (one: Wired) => void;
}

const button =
  "shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint hover:border-accent hover:text-accent";

const pillSaid = (one: Wired): Word => {
  if (hooksAmiss(one)) return one.hooks?.gone ? "hooksLost" : "hooksAstray";
  if (one.hooks?.on) return "hooksOn";
  return one.hooks?.plugged ? "hooksPlugged" : "hooksOff";
};

const pillDress = (one: Wired): string => {
  if (hooksAmiss(one)) return "border-hue-amber/40 text-hue-amber";
  return one.hooks?.on || one.hooks?.plugged
    ? "border-hue-green/40 text-hue-green"
    : "border-line text-soft";
};

export default function Hooking({ one, held, onHook, onFix }: Readonly<Props>) {
  const [asking, setAsking] = useState(false);
  const hooks = one.hooks;
  if (!hooks) return null;
  const byPlugin = !!hooks.plugged && !hooks.on;
  const broken = hooksAmiss(one);
  return (
    <div className="px-3 py-2">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
        <span className="min-w-[9rem] flex-1 text-[12.5px] text-soft">{t("hooksLabel")}</span>
        <span
          className={`shrink-0 rounded-full border px-2 py-0.5 text-[11.5px] whitespace-nowrap ${pillDress(one)}`}
        >
          {t(pillSaid(one))}
        </span>
        {broken && onFix && (
          <button
            type="button"
            disabled={held}
            aria-label={`${t("wiringFix")} · ${t("hooksLabel")}`}
            onClick={() => onFix(one)}
            className={`${button} border-accent text-accent`}
          >
            {t("wiringFix")}
          </button>
        )}
        {!byPlugin && (
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
        )}
      </div>
      {broken && hooks.points && (
        <p className="mt-1 text-[12.5px] text-soft [overflow-wrap:anywhere]">
          {fill(hooks.gone ? "wiringPointsGone" : "wiringPoints", hooks.points)}
        </p>
      )}
      {hooks.plugged && hooks.on && (
        <p className="mt-2 text-[12.5px] leading-relaxed text-urgent">{t("hooksTwice")}</p>
      )}
      {asking && !hooks.on && !byPlugin && (
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
