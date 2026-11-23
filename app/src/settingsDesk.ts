import { useEffect, useState } from "react";
import type { Checked, Item } from "./attention";
import { useKnownAgents } from "./knownAgents";

export function useSettingsDesk() {
  const [attention, setAttention] = useState<Item[] | null>(null);
  const [finding, setFinding] = useState("");
  const [lastChecked, setLastChecked] = useState<Checked | null>(null);
  const agents = useKnownAgents();
  const { lookIfOpen } = agents;
  useEffect(() => {
    lookIfOpen().catch(() => undefined);
  }, [lookIfOpen]);
  return { attention, setAttention, finding, setFinding, lastChecked, setLastChecked, agents };
}

export type SettingsDesk = ReturnType<typeof useSettingsDesk>;
