import type { AgentsState } from "../agentsSync";
import { fill, t } from "../locales";
import { button, primary } from "./AgentsReview";
import Modal from "./Modal";

export const foundSaid = (state: AgentsState): string => {
  const names = state.assistants.map((one) => one.name);
  if (names.length === 0) return t("agentsFoundNone");
  const last = names[names.length - 1];
  const listed =
    names.length === 1 ? last : `${names.slice(0, -1).join(", ")} ${t("agentsAnd")} ${last}`;
  return fill("agentsFoundHere", listed);
};

/// What the notice and the settings card both say about an AGENTS that is off.
export function AgentsFound({ state }: Readonly<{ state: AgentsState }>) {
  return (
    <>
      <p className="text-[13px] leading-relaxed text-ink">{foundSaid(state)}</p>
      <p className="mt-1 text-[11.5px] text-faint">{t("agentsFoundHow")}</p>
    </>
  );
}

interface Props {
  state: AgentsState;
  busy: boolean;
  trouble?: string;
  onLight: () => void;
  onLater: () => void;
  onHide: () => void;
}

export default function AgentsAsleep({
  state,
  busy,
  trouble,
  onLight,
  onLater,
  onHide,
}: Readonly<Props>) {
  return (
    <Modal title="AGENTS" onClose={onLater}>
      <p className="mb-3 text-[12.5px] leading-relaxed text-soft">{t("agentsAsleepLead")}</p>
      <AgentsFound state={state} />
      {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
      <div className="mt-5 flex items-center gap-2 text-[12.5px]">
        <button type="button" onClick={onHide} className={`${button} mr-auto`}>
          {t("agentsHide")}
        </button>
        <button type="button" onClick={onLater} className={button}>
          {t("agentsNotNow")}
        </button>
        <button type="button" disabled={busy} onClick={onLight} className={primary}>
          {t("agentsLight")}
        </button>
      </div>
    </Modal>
  );
}
