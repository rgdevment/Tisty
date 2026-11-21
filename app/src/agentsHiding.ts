import { AGENTS_CHANGED, type AgentsStage, agentsTurn } from "./agentsSync";
import { hide, hideChoice, isHidden } from "./hiding";
import { t } from "./locales";
import { sure } from "./sure";
import type { Choice } from "./ui/Menu";

/// Agents is never hidden and syncing, so an unknown stage asks too.
export const hideAgents = async (keys: string[], stage: AgentsStage | null): Promise<boolean> => {
  if (stage !== "brought" && stage !== "unbrought") {
    const yes = await sure(t("agentsHideFirst"), { yes: t("agentsHideStop"), no: t("cancel") });
    if (!yes) return false;
    await agentsTurn(null, false);
    window.dispatchEvent(new Event(AGENTS_CHANGED));
  }
  hide(keys[0]);
  return true;
};

interface Told {
  changed: () => void;
  fail: (problem: unknown) => void;
}

export const agentsHideChoice = (
  keys: string[],
  stage: AgentsStage | null,
  told: Told,
  off = false,
): Choice => {
  const plain = hideChoice(keys, off);
  if (keys.some(isHidden)) return plain;
  return {
    ...plain,
    onPick: () =>
      hideAgents(keys, stage)
        .then((hidden) => hidden && told.changed())
        .catch(told.fail),
  };
};
