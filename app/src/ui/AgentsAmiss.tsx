import { hushAmiss, hushedHere, onlyHooks, unheeded } from "../agentsAmiss";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";
import { fill, t } from "../locales";

export const useUnheeded = (known: KnownAgents | undefined): Wired[] => {
  const hushed = hushedHere.use().said;
  return unheeded(known?.agent?.on ? known.agents : null, hushed);
};

const saidOf = (amiss: Wired[]): string => {
  if (amiss.length > 1) return fill("amissMany", String(amiss.length));
  return fill(onlyHooks(amiss[0]) ? "amissHooks" : "amissOne", amiss[0].name);
};

export default function AgentsAmiss({
  amiss,
  onOpen,
}: Readonly<{ amiss: Wired[]; onOpen: () => void }>) {
  if (amiss.length === 0) return null;
  return (
    <div
      role="status"
      className="mx-2.5 mb-2 flex flex-col gap-1.5 rounded-[10px] border border-hue-amber/40 bg-sheet px-3 py-2.5"
    >
      <span className="flex items-center gap-2 text-[12.5px] font-semibold">
        <span aria-hidden="true" className="h-2 w-2 shrink-0 rounded-full bg-hue-amber" />
        {saidOf(amiss)}
      </span>
      <span className="text-[11.5px] leading-relaxed text-soft">{t("amissWhy")}</span>
      <span className="mt-0.5 flex items-center gap-2">
        <button
          type="button"
          onClick={onOpen}
          className="rounded-md bg-accent px-3 py-1 text-[12.5px] font-semibold text-bg"
        >
          {t("amissSee")}
        </button>
        <button
          type="button"
          onClick={() => hushAmiss(amiss)}
          className="px-1.5 py-1 text-[12.5px] text-faint hover:text-ink"
        >
          {t("agentsNotNow")}
        </button>
      </span>
    </div>
  );
}
