import { useEffect } from "react";
import type { TistySkills } from "../agentsSync";
import { attention, firstSteps, type Item, type Known, tiles, whole } from "../attention";
import type { About, Carrying, CloudStatus, Ready, Reviewed, Settings, Wired } from "../core";
import { cloudNamed } from "../locales";
import type { SettingsDesk } from "../settingsDesk";
import { leaf } from "./Mode";
import { told } from "./SyncFoot";
import { checkedOf, type Seen, waitingIn } from "./Upkeep";
import { useFoot } from "./useFoot";

interface Read {
  state: Carrying | null;
  status: CloudStatus | null;
  build: About | null;
  kept: Settings | null;
  keys: string | null | undefined;
  alias: string | null | undefined;
  ready: Ready | null;
  audit: Reviewed | null;
  seen: Seen | null;
  wired: Wired[] | null;
  skills: TistySkills | null;
  waiting: boolean;
  desk?: SettingsDesk;
}

const said = <T>(one: T | null): string | null => (one === null ? null : JSON.stringify(one));
const unsaid = <T>(one: string | null): T | null => (one === null ? null : (JSON.parse(one) as T));

export function useOverview(read: Read) {
  const { state, status, audit, seen, desk } = read;
  const { foot } = useFoot(state, status);
  const own = audit && seen ? checkedOf(audit, seen) : null;
  const known: Known = {
    ...read,
    foot,
    checked: own ?? desk?.lastChecked ?? null,
    machines: audit ? waitingIn(audit) : null,
  };
  const items = attention(known);
  const named = !state
    ? ""
    : state.mode === "cloud"
      ? cloudNamed(status?.provider ?? state.cloudProvider)
      : leaf(state.chosen ?? "");
  const line = foot && state ? told(foot, state, named).line : undefined;

  const ownSaid = said(own);
  const keepChecked = desk?.setLastChecked;
  useEffect(() => {
    const now = unsaid<Known["checked"]>(ownSaid);
    if (now) keepChecked?.(now);
  }, [ownSaid, keepChecked]);

  const itemsSaid = said(items);
  const publish = desk?.setAttention;
  useEffect(() => publish?.(unsaid<Item[]>(itemsSaid)), [itemsSaid, publish]);
  const forget = desk?.setFinding;
  useEffect(
    () => () => {
      publish?.(null);
      forget?.("");
    },
    [publish, forget],
  );

  return {
    items,
    whole: whole(known),
    steps: firstSteps(known),
    tiles: tiles(known, { named, line }),
  };
}
