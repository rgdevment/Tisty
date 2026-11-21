import { useState } from "react";
import type { Checked, Item } from "./attention";

export function useSettingsDesk() {
  const [attention, setAttention] = useState<Item[] | null>(null);
  const [finding, setFinding] = useState("");
  const [lastChecked, setLastChecked] = useState<Checked | null>(null);
  return { attention, setAttention, finding, setFinding, lastChecked, setLastChecked };
}

export type SettingsDesk = ReturnType<typeof useSettingsDesk>;
