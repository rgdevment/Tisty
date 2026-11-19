import type { Act, Panel } from "tisty-editor/read";
import { type AgentDoc, extensionOf, type Place } from "../agentDocs";
import { stamped } from "../format";
import { valuesIn } from "../front";
import { fill, t } from "../locales";

type Facts = Extract<Panel, { kind: "facts" }>;

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
  declined: "agentFileSyncsDeclined",
  notAgreed: "agentFileSyncsNotAgreed",
  placedByAgent: "agentFileSyncsPlacedByAgent",
  otherMemory: "agentFileSyncsOtherMemory",
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

const backupsOf = (places: readonly Place[]): string => {
  const count = places.reduce((sum, one) => sum + one.kept.length, 0);
  if (count === 0) return t("agentFileNone");
  return count === 1 ? t("agentBackupsOne") : fill("agentBackups", String(count));
};

export const agentFacts = (doc: AgentDoc, body: string, acts: readonly Act[]): Facts => {
  const single = doc.places.length === 1;
  const wrote = latest(doc.places);
  const original = doc.places.some((one) => one.kept.some((backup) => backup.pinned));
  return {
    kind: "facts",
    label: t("agentFileTitle"),
    rows: [
      [t("agentFileReads"), doc.places.map((one) => one.named).join(", ") || t("agentFileNobody")],
      ...doc.places.map(
        (one) => [single ? t("agentFileAt") : fill("agentFileAtFor", one.named), one.at] as const,
      ),
      [t("agentFileFormat"), formatOf(doc, body)],
      [t("agentFileWrote"), wrote ? stamped(wrote) : ""],
      [t("agentFileBackups"), backupsOf(doc.places)],
      [t("agentFileOriginal"), t(original ? "agentFileOriginalKept" : "agentFileOriginalNotYet")],
      [t("agentFileSyncs"), syncsOf(doc)],
    ],
    acts,
  };
};
