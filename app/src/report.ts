import type { Facts } from "./core";
import { stamped, weigh } from "./format";
import { fill, t } from "./locales";

type Line = [string, string];

/// Worded here, not in Rust: the window already speaks both languages.
export function written(facts: Facts, now = new Date()): string {
  const said: string[] = [
    `# ${t("reportHead")}`,
    `${t("reportMade")}  ${stamped(now.toISOString(), now)}`,
    "",
    t("reportStays"),
  ];

  const block = (name: string, lines: Line[]) => {
    said.push("", `[${name}]`, ...aligned(lines));
  };

  block(t("reportBuild"), [
    [t("wordVersion"), facts.version],
    [t("wordChannel"), t(facts.dev ? "wordDev" : "wordRelease")],
    ...(facts.sandbox ? ([[t("wordSandbox"), facts.sandbox]] as Line[]) : []),
    [t("wordLocale"), facts.locale],
    [t("wordZone"), facts.zone],
  ]);

  block(t("reportSystem"), [
    [t("wordOs"), facts.os],
    [t("wordArch"), facts.arch],
    [t("wordWebview"), facts.webview ?? "?"],
  ]);

  block(t("reportStore"), [
    [t("wordPath"), facts.store],
    [t("wordDevices"), String(facts.devices)],
    [t("wordEvents"), String(facts.events)],
    [
      t("wordTasks"),
      `${fill("openTasks", String(facts.open))} · ${fill("archivedTasks", String(facts.archived))}`,
    ],
    [t("wordLists"), named(facts.lists, facts.listNames, "lista")],
    [t("wordTags"), named(facts.tags, facts.tagNames, "tag")],
    [t("wordCache"), t(CACHE[facts.cache])],
    [
      t("wordAttachments"),
      `${facts.attachments} · ${weigh(facts.attachmentBytes)} · ${facts.loose} ${t("wordLoose")} (${weigh(facts.looseBytes)})`,
    ],
    [t("wordWeight"), weigh(facts.weight)],
  ]);

  block(t("reportSettings"), [
    [t("wordSyncs"), facts.syncs ? t(facts.shared ? "yesShared" : "yesLocal") : t("noSync")],
    [
      t("wordBackup"),
      facts.shared
        ? t("backupIsShared")
        : `${t("backupAvailable")} · ${facts.backedUpAt ? stamped(facts.backedUpAt, now) : t("backupNever")}`,
    ],
    [t("wordNotices"), speaking(facts.quiet)],
    [t("wordCopiesUpTo"), weigh(facts.attachUpTo)],
    [t("wordTerminal"), t(facts.inPath ? "inThePath" : "notInThePath")],
    [t("wordShortcut"), facts.shortcut ?? t("nothingBound")],
  ]);

  return `${said.join("\n")}\n`;
}

const CACHE = {
  agrees: "cacheAgrees",
  stale: "cacheStale",
  diverged: "cacheDiverged",
  none: "cacheNone",
} as const;

/// The count and the position are what a filter fault needs; the words are not.
function named(count: number, names: string[], kind: string): string {
  if (count === 0) return "0";
  const shown = names.length > 0 ? names : Array.from({ length: count }, (_, at) => `${kind}#${at + 1}`);
  return `${count}   (${shown.join(", ")})`;
}

/// The store keeps the muted ones; listing those would read as what works.
function speaking(quiet: string[]): string {
  const on = ["screen", "chime"].filter((one) => !quiet.includes(one));
  return on.length === 0 ? t("wordNone") : on.map((one) => t(WORDED[one])).join(", ");
}

const WORDED: Record<string, "noticeScreen" | "noticeChime"> = {
  screen: "noticeScreen",
  chime: "noticeChime",
};

function aligned(lines: Line[]): string[] {
  const width = Math.max(...lines.map(([label]) => label.length));
  return lines.map(([label, value]) => `${label.padEnd(width + 2)}${value}`);
}
