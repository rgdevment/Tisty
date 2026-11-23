import { amiss, lost, onlyHooks, skillsBehind } from "./agentsAmiss";
import type { TistySkills } from "./agentsSync";
import type { About, Carrying, CloudStatus, Ready, Settings, Wired } from "./core";
import { type Footing, synced } from "./footing";
import { daysFrom, stamped } from "./format";
import { fill, t } from "./locales";
import type { Spot } from "./settingsRows";
import type { Tab } from "./views";

export interface Item {
  key: string;
  spot: Spot;
  said: string;
  act: string;
  why?: string;
}

export interface Checked {
  many: number;
  at: string;
  missed?: string;
  waiting: string[];
}

export interface Known {
  state: Carrying | null;
  foot: Footing | null;
  build: About | null;
  kept: Settings | null;
  keys: string | null | undefined;
  alias: string | null | undefined;
  ready: Ready | null;
  checked: Checked | null;
  wired: Wired[] | null;
  skills?: TistySkills | null;
  waiting?: boolean;
  machines?: string[] | null;
  status?: CloudStatus | null;
}

const waitingOf = (known: Known): string[] => known.machines ?? known.checked?.waiting ?? [];

export const QUIET_DAYS = 3;
export const BACKUP_DAYS = 30;

const SYNCING: Spot = { page: "sync", row: "syncing" };

export const spotOf = (foot: Footing): Spot =>
  foot.kind === "sealing" ? { page: "security", row: "sealing" } : SYNCING;

const NEEDS_YOU = {
  lost: "attSyncLost",
  look: "attSyncLook",
  paused: "attSyncPaused",
} as const;

export const quiet = (state: Carrying, now = new Date()): boolean =>
  Boolean(state.chosen && state.heard && -daysFrom(state.heard, now) >= QUIET_DAYS);

const stale = (state: Carrying, now: Date): boolean =>
  Boolean(state.backedUpAt && -daysFrom(state.backedUpAt, now) >= BACKUP_DAYS);

function syncItems(state: Carrying, foot: Footing | null, now: Date): Item[] {
  const asks =
    foot && foot.kind in NEEDS_YOU ? NEEDS_YOU[foot.kind as keyof typeof NEEDS_YOU] : null;
  if (asks) return [{ key: "sync", spot: SYNCING, said: t(asks), act: t("attSee") }];
  if (quiet(state, now))
    return [
      {
        key: "quiet",
        spot: SYNCING,
        said: t("attSyncQuiet"),
        act: t("attSee"),
        why: t("attSyncQuietWhy"),
      },
    ];
  return [];
}

function updateItem(ready: Ready | null): Item[] {
  if (!ready) return [];
  const said = ready.version ? fill("updateThere", ready.version) : t("updateOffered");
  const act = t(ready.installs ? "updateInstall" : "attSee");
  return [{ key: "update", spot: { page: "about", row: "update" }, said, act }];
}

// Null while a read the page makes on opening is still out; a read that failed only leaves its items out.
export function attention(known: Known, now = new Date()): Item[] | null {
  const { state, kept, build, checked } = known;
  if (known.waiting || !state) return null;
  const items: Item[] = [...updateItem(known.ready), ...syncItems(state, known.foot, now)];
  if (build?.standing?.phrase === "loose")
    items.push({
      key: "phrase",
      spot: { page: "security", row: "identity" },
      said: t("attPhrase"),
      act: t("attPhraseAct"),
      why: t("attPhraseWhy"),
    });
  for (const name of waitingOf(known))
    items.push({
      key: `machine-${name}`,
      spot: { page: "security", row: "machines" },
      said: fill("attMachine", name),
      act: t("attMachineAct"),
      why: t("attMachineWhy"),
    });
  if (state.backedUpAt && stale(state, now))
    items.push({
      key: "backup",
      spot: { page: "copies", row: "backup" },
      said: fill("attBackupOld", stamped(state.backedUpAt, now)),
      act: t("attBackupAct"),
    });
  if (checked && checked.many > 0)
    items.push({
      key: "health",
      spot: { page: "health", row: "review" },
      said: checked.many === 1 ? t("attHealthOne") : fill("attHealthMany", String(checked.many)),
      act: t("attHealthAct"),
    });
  if (known.keys === null && kept && kept.quickKeys !== "off")
    items.push({
      key: "keys",
      spot: { page: "notices", row: "quick" },
      said: t("attKeys"),
      act: t("attKeysAct"),
      why: t("attKeysWhy"),
    });
  return [...items, ...agentItems(known)];
}

const MENDING: Spot = { page: "agents", shelf: "wired" };

const amissSaid = (all: Wired[], gone: boolean): string => {
  const [one] = all;
  if (all.length > 1) return fill(gone ? "attAstrayMany" : "attElsewhereMany", String(all.length));
  if (onlyHooks(one)) return fill(gone ? "attAstrayHooks" : "attElsewhereHooks", one.name);
  return fill(gone ? "attAstray" : "attElsewhere", one.name);
};

function agentItems({ wired, skills }: Known): Item[] {
  const all = wired?.filter(amiss) ?? [];
  const item = (key: string, said: string): Item => ({
    key,
    spot: MENDING,
    said,
    act: t("attAstrayAct"),
  });
  const items = [true, false].flatMap((gone) => {
    const group = all.filter((one) => lost(one) === gone);
    return group.length > 0
      ? [item(gone ? "agents-lost" : "agents-elsewhere", amissSaid(group, gone))]
      : [];
  });
  return skillsBehind(skills) ? [...items, item("agents-skills", t("attSkillsBehind"))] : items;
}

// «Nothing asks for you» is said only when every source answered.
export const whole = (known: Known): boolean =>
  Boolean(known.state && known.kept && known.build && known.keys !== undefined);

export const counted = (items: Item[] | null): Partial<Record<Tab, number>> => {
  const many: Partial<Record<Tab, number>> = {};
  for (const one of items ?? []) many[one.spot.page] = (many[one.spot.page] ?? 0) + 1;
  return many;
};

export interface Step {
  key: string;
  spot: Spot;
  said: string;
  why: string;
  done: boolean;
  optional?: boolean;
}

// The four that are not optional decide when the steps go; staying on this computer counts as choosing.
export function firstSteps(known: Known): Step[] | null {
  const { state, build, alias } = known;
  const phrase = build?.standing?.phrase;
  if (!state || !phrase || alias === undefined) return null;
  return [
    {
      key: "sync",
      spot: SYNCING,
      said: t("firstSync"),
      why: t("firstSyncWhy"),
      done: state.asked,
    },
    {
      key: "phrase",
      spot: { page: "security", row: "identity" },
      said: t("firstPhrase"),
      why: t("firstPhraseWhy"),
      done: phrase !== "none",
    },
    {
      key: "backup",
      spot: { page: "copies", row: "backup" },
      said: t("firstBackup"),
      why: t("firstBackupWhy"),
      done: Boolean(state.backedUpAt),
    },
    {
      key: "alias",
      spot: { page: "writing", row: "alias" },
      said: t("firstAlias"),
      why: t("firstAliasWhy"),
      done: Boolean(alias),
    },
    {
      key: "agent",
      spot: { page: "agents", shelf: "wired", row: "wired" },
      said: t("firstAgent"),
      why: t("firstAgentWhy"),
      done: Boolean(known.wired?.some((one) => one.wired)),
      optional: true,
    },
  ];
}

export const stepsShown = (steps: Step[] | null): boolean =>
  Boolean(steps?.some((one) => !one.optional && !one.done));

export type Dot = "ok" | "warn" | "off";

export interface Tile {
  page: Tab;
  dot: Dot;
  head: string;
  detail: string;
}

const tile = (page: Tab, dot: Dot, head: string, detail: string): Tile => ({
  page,
  dot,
  head,
  detail,
});

function syncTile(known: Known, named: string, line: string | undefined, now: Date): Tile | null {
  const { state, foot } = known;
  if (!state) return null;
  if (state.mode === "later")
    return tile("sync", "warn", t("tileSyncLater"), t("tileSyncLaterWhy"));
  if (!synced(state)) return tile("sync", "off", t("tileSyncOff"), t("tileSyncHere"));
  const head = fill("tileSyncIn", named);
  if (state.mode === "cloud" && !known.status)
    return tile("sync", "off", head, t("tileSyncUnread"));
  if (quiet(state, now) && state.heard)
    return tile("sync", "warn", head, fill("tileSyncQuiet", stamped(state.heard, now)));
  return tile("sync", footDot(foot), head, line ?? t("tileSyncHere"));
}

// Only a round that ended well, or one under way, is green; what passes by itself stays neutral.
const footDot = (foot: Footing | null): Dot =>
  !foot
    ? "off"
    : foot.kind in NEEDS_YOU
      ? "warn"
      : foot.kind === "ok" || foot.kind === "syncing"
        ? "ok"
        : "off";

function copiesTile({ state }: Known, now: Date): Tile | null {
  if (!state) return null;
  if (!state.backedUpAt) return tile("copies", "off", t("tileBackupNone"), t("backupNever"));
  return tile(
    "copies",
    stale(state, now) ? "warn" : "ok",
    t("tileBackup"),
    stamped(state.backedUpAt, now),
  );
}

const PHRASE = {
  none: ["off", "tilePhraseNone", "tilePhraseNoneWhy"],
  loose: ["warn", "tilePhraseLoose", "tilePhraseLooseWhy"],
  ok: ["ok", "tilePhraseOk", "tilePhraseOkWhy"],
  elsewhere: ["warn", "tilePhraseElsewhere", "tilePhraseElsewhereWhy"],
} as const;

function securityTile(known: Known): Tile | null {
  const phrase = known.build?.standing?.phrase;
  if (!phrase) return null;
  const [dot, head, why] = PHRASE[phrase];
  return tile("security", waitingOf(known).length ? "warn" : dot, t(head), t(why));
}

function agentsTile({ build, wired, skills }: Known): Tile | null {
  const standing = build?.standing;
  if (!standing) return null;
  if (!standing.door) return tile("agents", "off", t("tileDoorShut"), t("tileDoorShutWhy"));
  const many = (wired?.filter(amiss).length ?? 0) + (skillsBehind(skills) ? 1 : 0);
  if (many > 0)
    return tile("agents", "warn", t("tileDoorOpen"), fill("tileAgentsFix", String(many)));
  return tile("agents", "ok", t("tileDoorOpen"), fill("tileDoorAs", standing.door));
}

function healthTile({ checked }: Known, now: Date): Tile {
  if (!checked) return tile("health", "off", t("tileHealthUnknown"), t("tileHealthUnknownWhy"));
  const head =
    checked.many === 0
      ? t(checked.missed ? "reviewPartly" : "tileHealthOk")
      : checked.many === 1
        ? t("healthOne")
        : fill("healthMany", String(checked.many));
  return tile(
    "health",
    checked.many === 0 && !checked.missed ? "ok" : "warn",
    head,
    fill("tileChecked", stamped(checked.at, now)),
  );
}

// Only an answer from today says up to date; none yet, an older one, or one stamped ahead of this clock does not.
export const heardToday = (looked: string | null | undefined, now = new Date()): boolean =>
  Boolean(looked) && daysFrom(looked as string, now) === 0;

const lookedTile = (looked: string | null | undefined, now: Date): [Dot, string] =>
  !looked
    ? ["off", t("tileNotLooked")]
    : heardToday(looked, now)
      ? ["ok", t("tileUpToDate")]
      : ["off", fill("tileLooked", stamped(looked, now))];

function aboutTile({ build, ready }: Known, now: Date): Tile | null {
  if (!build) return null;
  if (ready)
    return tile(
      "about",
      "warn",
      ready.version ? fill("tileUpdate", ready.version) : t("updateOffered"),
      fill("tileYouHave", build.version),
    );
  const [dot, detail] = lookedTile(build.standing?.looked, now);
  return tile("about", dot, fill("tileVersion", build.version), detail);
}

export const TILE_PAGES: Tab[] = ["sync", "copies", "security", "agents", "health", "about"];

// A tile whose source has not arrived stays null and is drawn neutral, never as a false value.
export function tiles(
  known: Known,
  sync: { named: string; line?: string },
  now = new Date(),
): Record<string, Tile | null> {
  return {
    sync: syncTile(known, sync.named, sync.line, now),
    copies: copiesTile(known, now),
    security: securityTile(known),
    agents: agentsTile(known),
    health: healthTile(known, now),
    about: aboutTile(known, now),
  };
}
