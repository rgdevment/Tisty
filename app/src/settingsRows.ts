import { t } from "./locales";
import { GROUPS, pageOf } from "./settingsPages";
import type { Chosen, Tab } from "./views";

type Word = Parameters<typeof t>[0];

export type Shelf = "door" | "wired" | "folder" | "credit" | "can";

export interface Row {
  page: Tab;
  shelf?: Shelf;
  title: Word;
  why?: Word;
  words?: Word[];
}

const row = (page: Tab, title: Word, why?: Word, words?: Word[]): Row => ({
  page,
  title,
  why,
  words,
});
const agents = (shelf: Shelf, title: Word, why?: Word, words?: Word[]): Row => ({
  ...row("agents", title, why, words),
  shelf,
});

export const ROWS = {
  tongue: row("window", "tongue", "tongueWhy"),
  look: row("window", "look", "lookWhy"),
  beaver: row("window", "beaverPeeks", "beaverPeeksWhy"),
  closing: row("window", "closingSetting", "closingSettingWhy"),
  wake: row("window", "wake", "wakeOn"),
  hidden: row("window", "hiddenTitle", "hiddenLead"),
  quick: row("notices", "quick", undefined, ["quickKeys", "findKeyboard"]),
  idea: row("notices", "quickIdea", undefined, ["quickIdeaKeys", "findKeyboard"]),
  screen: row("notices", "noticeScreen", "noticesWhy"),
  chime: row("notices", "noticeChime"),
  urgent: row("notices", "noticeUrgent"),
  calendar: row("notices", "calendarSetting", "calendarSettingWhy"),
  alias: row("writing", "alias", "aliasShort"),
  signName: row("writing", "signName", "signNameShort"),
  earlier: row("writing", "earlierLimit", undefined, ["earlierBand"]),
  syncing: row("sync", "whereItSyncs", undefined, [
    "syncNow",
    "turnSyncOn",
    "revealFolder",
    "changeFolder",
    "moveOpen",
    "syncAgain",
  ]),
  syncMachines: row("sync", "theMachines", undefined, ["seeInSecurity"]),
  syncOff: row("sync", "syncOffTitle", "syncOffWhy"),
  cloudForget: row("sync", "cloudForgetHere", "cloudForgetWhy"),
  cloudRevoke: row("sync", "cloudRevokeAll", "cloudRevokeWhy"),
  cloudAbandon: row("sync", "cloudAbandon", "cloudAbandonWhy"),
  identity: row("security", "tabIdentity", "identityWhat"),
  machines: row("security", "theMachines", "machinesWhat"),
  sealing: row("security", "sealTitle"),
  sealWipe: row("security", "sealWipe", "sealWipeWhy"),
  sealOff: row("security", "sealOff", "sealOffWhy"),
  backup: row("copies", "backupSave", "backupWhat", ["backupMake"]),
  parcels: row("copies", "parcelsTitle", "parcelsWhy"),
  restore: row("copies", "restoreTitle", "restoreWhat"),
  leave: row("copies", "leaveTitle", "leaveWhy"),
  folder: row("storage", "aboutStore", undefined, ["whereItLives"]),
  weighs: row("storage", "storageWeighs"),
  holds: row("storage", "holdsTitle", undefined, ["attachTitle"]),
  unused: row("storage", "freeUnusedTitle", undefined, ["freeTitle"]),
  loose: row("storage", "looseAre", undefined, ["freeTitle"]),
  twins: row("storage", "twinsAre", "twinsWhat"),
  door: agents("door", "agentsTitle"),
  asks: agents("door", "agentsAsk", "agentsAskWhy"),
  hooks: agents("door", "agentsHooks", "agentsHooksWhy"),
  wired: agents("wired", "wiringTitle"),
  agentsFolder: agents("folder", "agentsSettingsTitle"),
  credit: agents("credit", "creditTitle"),
  creditMark: agents("credit", "creditMark", "creditMarkWhy"),
  creditRows: agents("credit", "creditRows", "creditRowsWhy"),
  creditInside: agents("credit", "creditInside", "creditInsideWhy"),
  can: agents("can", "agentsTabCan"),
  review: row("health", "reviewAgain", "leadHealth"),
  index: row("health", "quickIndex", "quickIndexWhy"),
  report: row("health", "reportTitle", "reportLine"),
  update: row("about", "lookNow", undefined, ["updateInstall"]),
  candidates: row("about", "betaTake", "betaWarns"),
  greet: row("about", "greetAgain", "greetAgainWhy", ["welcomeGuide"]),
  site: row("about", "helpOnSite", "helpOnSiteWhy"),
  support: row("about", "supportTitle", "supportWhy"),
  tools: row("about", "otherTools"),
} satisfies Record<string, Row>;

export type RowId = keyof typeof ROWS;

export const ROW_IDS = Object.keys(ROWS) as RowId[];

export interface Spot {
  page: Tab;
  shelf?: Shelf;
  row?: RowId;
}

export const landedOn = (spot: Spot): Chosen => ({
  named: "keeping",
  tab: spot.page,
  shelf: spot.shelf,
  row: spot.row,
  landedAt: Date.now(),
});

export const settingAt = (id: RowId) => ({ id: `setting-${id}`, "data-setting": id });

const plain = (said: string): string =>
  said
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();

const spoken = (one: Row): string =>
  plain(
    [one.title, one.why, ...(one.words ?? [])].flatMap((key) => (key ? [t(key)] : [])).join(" "),
  );

const PAGE_ORDER: Tab[] = GROUPS.flatMap((group) => group.pages);

export interface Hit {
  id: RowId;
  row: Row;
  title: string;
  page: string;
}

export const found = (asked: string): Hit[] => {
  const words = plain(asked).split(" ").filter(Boolean);
  if (words.length === 0) return [];
  return ROW_IDS.filter((id) => {
    const said = spoken(ROWS[id]);
    return words.every((word) => said.includes(word));
  })
    .sort((one, other) => PAGE_ORDER.indexOf(ROWS[one].page) - PAGE_ORDER.indexOf(ROWS[other].page))
    .map((id) => ({
      id,
      row: ROWS[id],
      title: t(ROWS[id].title),
      page: t(pageOf(ROWS[id].page).label),
    }));
};
