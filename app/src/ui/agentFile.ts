import { type AgentDoc, extensionOf, type Place } from "../agentDocs";
import { stamped, weigh } from "../format";
import { valuesIn } from "../front";
import { fill, t } from "../locales";

export type Tone = "amber" | "red";

export type FactRow =
  | { readonly kind: "said"; readonly label: string; readonly value: string; readonly tone?: Tone }
  | { readonly kind: "path"; readonly label: string; readonly at: string; readonly note?: string }
  | { readonly kind: "backups"; readonly label: string; readonly place: Place }
  | { readonly kind: "own"; readonly label: string; readonly value: string }
  | { readonly kind: "raw"; readonly value: string };

// Codex reads AGENTS.md only up to this many bytes and leaves the rest unread.
export const CODEX_CAP = 32 * 1024;
const CODEX_NEAR = 28 * 1024;

export const formatOf = (doc: AgentDoc, body: string): string => {
  const [address = "", ...flags] = valuesIn(body, "agent") ?? [];
  const ending = extensionOf(address)?.toUpperCase();
  const kind = doc.code ? (ending ?? t("agentFilePlain")) : "Markdown";
  const ends = flags.includes("crlf")
    ? "CRLF"
    : flags.includes("as-is")
      ? t("agentFileMixedEnds")
      : "LF";
  const more = [
    flags.includes("bom") && "BOM",
    flags.includes("open-end") && t("agentFileOpenEnd"),
  ];
  return [kind, ends, ...more].filter(Boolean).join(" · ");
};

const NOT_HERE = {
  waiting: "agentFileSyncsWaiting",
  approvedWaiting: "agentFileSyncsWaiting",
  declined: "agentFileSyncsDeclined",
  notAgreed: "agentFileSyncsNotAgreed",
  placedByAgent: "agentFileSyncsPlacedByAgent",
  otherMemory: "agentFileSyncsOtherMemory",
  otherComputer: "agentFileSyncsOtherComputer",
  apart: "agentFileSyncsApart",
  contested: "agentFileSyncsContested",
  aside: "agentFileSyncsAside",
  hookWaiting: "agentHookWaiting",
  hookRegistered: "agentHookRegistered",
} as const;

const syncsOf = (doc: AgentDoc): string => {
  const said = standsOf(doc);
  return doc.proposal ? fill("agentFileSyncsProposal", said) : said;
};

const standsOf = (doc: AgentDoc): string => {
  if (doc.folderOff) return t("agentFileSyncsFolderOff");
  if (doc.here !== "applied" && doc.here !== "away") return t(NOT_HERE[doc.here]);
  const states = doc.places.map((one) => one.said.state);
  if (states.length === 0) return t("agentFileSyncsNowhere");
  if (states.every((one) => one === "noConsent")) return t("agentFileSyncsNotAllowed");
  const stopped = (one: Place) => one.said.state === "off" || one.said.state === "noConsent";
  if (doc.places.every(stopped)) return t("agentFileSyncsPaused");
  const paused = doc.places.filter(stopped).map((one) => one.named);
  if (paused.length > 0) return fill("agentFileSyncsBut", paused.join(", "));
  return t(doc.kind === "memory" ? "agentFileSyncsReads" : "agentFileSyncsYes");
};

const latest = (places: readonly Place[]): string | undefined =>
  places
    .map((one) => one.changed)
    .filter((one): one is string => one !== null)
    .sort((one, two) => Date.parse(two) - Date.parse(one))[0];

export const backupsSaid = (place: Place): string => {
  const count = place.kept.length;
  if (count === 0) return t("agentFileNone");
  const said = count === 1 ? t("agentBackupsOne") : fill("agentBackups", String(count));
  return place.kept.some((one) => one.pinned) ? fill("agentCopiesWithOriginal", said) : said;
};

const capped = (doc: AgentDoc, place: Place): boolean =>
  place.assistant === "codex" && doc.kind === "rule";

const sizeSaid = (doc: AgentDoc, place: Place, bytes: number): { value: string; tone?: Tone } => {
  const weighs = weigh(bytes);
  if (!capped(doc, place)) return { value: weighs };
  if (bytes > CODEX_CAP) return { value: fill("agentFileOverCap", weighs), tone: "red" };
  if (bytes >= CODEX_NEAR) return { value: fill("agentFileNearCap", weighs), tone: "amber" };
  return { value: weighs };
};

const composedOf = (place: Place): string | undefined => {
  if (place.shared === 2) return t("agentComposedOne");
  if (place.shared > 2) return fill("agentComposed", String(place.shared - 1));
  return undefined;
};

const eachPlace = (doc: AgentDoc): FactRow[] => {
  const single = doc.places.length === 1;
  const named = (alone: Parameters<typeof t>[0], many: Parameters<typeof fill>[0], place: Place) =>
    single ? t(alone) : fill(many, place.named);
  return doc.places.flatMap((place): FactRow[] => {
    const label = named("agentFileSize", "agentFileSizeFor", place);
    const size: FactRow[] =
      place.bytes === null ? [] : [{ kind: "said", label, ...sizeSaid(doc, place, place.bytes) }];
    return [
      {
        kind: "path",
        label: named("agentFileAt", "agentFileAtFor", place),
        at: place.at,
        note: composedOf(place),
      },
      ...size,
      { kind: "backups", label: named("agentFileBackups", "agentFileBackupsFor", place), place },
    ];
  });
};

const ownRows = (doc: AgentDoc): FactRow[] => [
  ...doc.own.fields.map(([key, value]): FactRow => ({ kind: "own", label: key, value })),
  ...(doc.own.raw === null ? [] : [{ kind: "raw" as const, value: doc.own.raw }]),
];

export const agentRows = (doc: AgentDoc, body: string): FactRow[] => {
  const wrote = latest(doc.places);
  const said = (label: Parameters<typeof t>[0], value: string): FactRow => ({
    kind: "said",
    label: t(label),
    value,
  });
  return [
    said("agentFileReads", doc.places.map((one) => one.named).join(", ") || t("agentFileNobody")),
    ...eachPlace(doc),
    said("agentFileFormat", formatOf(doc, body)),
    ...(wrote ? [said("agentFileWrote", stamped(wrote))] : []),
    said("agentFileSyncs", syncsOf(doc)),
    ...ownRows(doc),
  ];
};
