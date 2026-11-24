import { hush, hushedHere, unheeded } from "../agentsAmiss";
import { noticeSaid } from "../agentsSaid";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";
import { t } from "../locales";

export const useUnheeded = (known: KnownAgents | undefined): Wired[] => {
  const quiet = hushedHere.use().said;
  return unheeded(known?.agent?.on ? known.agents : null, "lost", quiet);
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
        {noticeSaid(amiss)}
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
          onClick={() => hush(amiss, "lost")}
          className="px-1.5 py-1 text-[12.5px] text-faint hover:text-ink"
        >
          {t("agentsNotNow")}
        </button>
      </span>
    </div>
  );
}
