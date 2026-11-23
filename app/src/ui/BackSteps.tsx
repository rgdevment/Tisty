import type { ReactNode } from "react";
import { t } from "../locales";
import { strong } from "./Rows";

interface Props {
  gone: boolean;
  locked: boolean;
  joined: boolean;
  held: boolean;
  onReconnect: () => void;
  onPhrase: () => void;
}

function Step({
  at,
  done,
  label,
  hint,
  act,
}: Readonly<{ at: number; done: boolean; label: string; hint?: string; act?: ReactNode }>) {
  return (
    <li className="flex flex-wrap items-center gap-2">
      <span
        aria-hidden="true"
        className={`w-4 shrink-0 text-center tabular-nums ${done ? "text-hue-teal" : "text-faint"}`}
      >
        {done ? "✓" : at}
      </span>
      <span className="min-w-[10rem] flex-1">
        <span className={done ? "text-soft" : "text-ink"}>{label}</span>
        {done && <span className="sr-only">{t("backDone")}</span>}
        {hint && <span className="block text-[11.5px] text-faint">{hint}</span>}
      </span>
      {act}
    </li>
  );
}

/** What this computer still needs to sync with a sealed cloud, in the order it can be given. */
export default function BackSteps({
  gone,
  locked,
  joined,
  held,
  onReconnect,
  onPhrase,
}: Readonly<Props>) {
  const button = (label: string, onPress: () => void) => (
    <button type="button" disabled={held} onClick={onPress} className={strong}>
      {label}
    </button>
  );
  const others = t(joined ? "sealLockedJoined" : "sealLockedNew");
  return (
    <div className="mt-2.5 text-[12.5px]">
      <p className="text-[11.5px] font-semibold text-faint">{t("backTitle")}</p>
      <ol className="mt-1.5 flex flex-col gap-2">
        <Step
          at={1}
          done={!gone}
          label={t(gone ? "backReconnect" : "backConnected")}
          act={gone && button(t("stateReconnectDo"), onReconnect)}
        />
        <Step
          at={2}
          done={!locked}
          label={t(locked ? "backPhrase" : "backKeyed")}
          hint={locked ? (gone ? t("backAfter") : others) : undefined}
          act={locked && !gone && button(t("enterButton"), onPhrase)}
        />
      </ol>
    </div>
  );
}
