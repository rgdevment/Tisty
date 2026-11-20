import type { ReactNode } from "react";
import { type AgentDoc, type Agreeing, fencedFile, type Here, type Place } from "../agentDocs";
import { copied } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { agentRows, backupsSaid, type FactRow } from "./agentFile";
import ComparedLines from "./ComparedLines";
import Glyph from "./Glyph";
import Modal from "./Modal";
import Page from "./Page";

export type Shown = { place: Place; backup: number | null; text: string };

export function Pill({
  tone,
  glyph,
  children,
}: {
  tone: string;
  glyph?: string;
  children: string;
}) {
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-[11.5px] ${tone}`}
    >
      {glyph && <Glyph name={glyph} className="h-[12px] w-[12px]" />}
      {children}
    </span>
  );
}

export function Action({
  glyph,
  label,
  onClick,
}: {
  glyph: string;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex items-center gap-1.5 rounded-md border border-line px-2.5 py-1 text-[12.5px] text-ink hover:bg-hover"
    >
      <Glyph name={glyph} className="h-[13px] w-[13px]" />
      {label}
    </button>
  );
}

const HERE_SAID = {
  waiting: "agentHereWaiting",
  approvedWaiting: "agentHereApprovedWaiting",
  declined: "agentHereDeclined",
  notAgreed: "agentHereNotAgreed",
  placedByAgent: "agentHerePlacedByAgent",
  otherMemory: "agentHereOtherMemory",
} as const;

// What keeps this computer from writing the document, and what the person can do about it here.
const HAND_SAID = {
  here: "agentHandHere",
  elsewhere: "agentHandElsewhere",
  agent: "agentHandAgent",
  earlier: "agentHandEarlier",
  outside: "agentHandOutside",
} as const;

// Approving takes in the words as they are now, so the person sees first what changed since the last approval and whose hand wrote it.
function Unagreed({ agreeing }: { agreeing: Agreeing }) {
  return (
    <div className="mt-1.5 text-ink">
      <p>{t(HAND_SAID[agreeing.hand])}</p>
      {agreeing.approved === "kept" ? (
        <ComparedLines
          lines={agreeing.lines}
          gone={t("agentProposalApproved")}
          came={t("agentAgreeNow")}
        />
      ) : (
        <p className="mt-1">
          {t(agreeing.approved === "never" ? "agentAgreeNever" : "agentAgreeElsewhere")}
        </p>
      )}
    </div>
  );
}

export function HereNote({
  here,
  agreeing,
  onTake,
  onAgree,
}: {
  here: Here;
  agreeing?: Agreeing | null;
  onTake: (here: boolean) => void;
  onAgree: () => void;
}) {
  if (here === "applied" || here === "away") return null;
  const waits = here === "waiting" || here === "approvedWaiting";
  return (
    <div
      role="status"
      className="mt-3 flex gap-2 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5 text-[12.5px] text-hue-amber"
    >
      <Glyph name="alert" className="mt-0.5 h-[15px] w-[15px] shrink-0" />
      <div className="min-w-0">
        <p className="font-medium">{t(HERE_SAID[here])}</p>
        {here === "notAgreed" && agreeing && <Unagreed agreeing={agreeing} />}
        <div className="mt-2 flex flex-wrap gap-2">
          {(waits || here === "declined") && (
            <Action glyph="done" label={t("agentApplyHere")} onClick={() => onTake(true)} />
          )}
          {waits && (
            <Action glyph="close" label={t("agentNotHere")} onClick={() => onTake(false)} />
          )}
          {here === "notAgreed" && (
            <Action glyph="done" label={t("agentAgree")} onClick={onAgree} />
          )}
        </div>
      </div>
    </div>
  );
}

function Link({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button type="button" className="text-accent hover:underline" onClick={onClick}>
      {label}
    </button>
  );
}

function Copy({ at, onError }: { at: string; onError: (e: unknown) => void }) {
  return (
    <button
      type="button"
      title={t("agentCopyPath")}
      aria-label={t("agentCopyPath")}
      onClick={() => void copied(at).catch(onError)}
      className="shrink-0 rounded-md p-0.5 text-faint hover:bg-hover hover:text-ink"
    >
      <Glyph name="copy" className="h-[13px] w-[13px]" />
    </button>
  );
}

const TONE = { amber: "text-hue-amber", red: "text-hue-red" } as const;

function Fact({
  row,
  onBackups,
  onError,
}: {
  row: FactRow;
  onBackups: (place: Place) => void;
  onError: (e: unknown) => void;
}) {
  if (row.kind === "raw")
    return (
      <pre className="m-0 whitespace-pre-wrap break-words border-b border-hair py-1.5 font-mono text-[11.5px] text-soft last:border-b-0">
        {row.value}
      </pre>
    );
  const small = row.kind === "own";
  return (
    <div className="flex items-baseline justify-between gap-4 border-b border-hair py-1.5 last:border-b-0">
      <dt className={`shrink-0 ${small ? "text-[11.5px] text-faint" : "text-soft"}`}>
        {row.label}
      </dt>
      <dd className={`m-0 min-w-0 text-right ${small ? "text-[11.5px] text-soft" : "text-ink"}`}>
        {row.kind === "path" && (
          <span className="flex items-center justify-end gap-1.5">
            <code className="min-w-0 truncate font-mono text-[11.5px]" title={row.at}>
              {row.at}
            </code>
            <Copy at={row.at} onError={onError} />
          </span>
        )}
        {row.kind === "path" && row.note && (
          <span className="block text-[11.5px] text-faint">{row.note}</span>
        )}
        {row.kind === "backups" && (
          <span>
            {backupsSaid(row.place)}
            {row.place.kept.length > 0 && (
              <>
                {" · "}
                <Link label={t("agentSee")} onClick={() => onBackups(row.place)} />
              </>
            )}
          </span>
        )}
        {(row.kind === "said" || row.kind === "own") && (
          <span className={row.kind === "said" && row.tone ? TONE[row.tone] : undefined}>
            {row.value}
          </span>
        )}
      </dd>
    </div>
  );
}

const keyOf = (row: FactRow): string => {
  if (row.kind === "raw") return "raw";
  if (row.kind === "path") return `path:${row.at}`;
  if (row.kind === "backups") return `backups:${row.place.at}`;
  return `${row.kind}:${row.label}`;
};

interface Facts {
  readonly doc: AgentDoc;
  readonly body: string;
  readonly children?: ReactNode;
  readonly onBackups: (place: Place) => void;
  readonly onError: (e: unknown) => void;
}

// The only place the file's facts are told: the page under it keeps only the words.
export function AgentFacts({ doc, body, onBackups, onError, children }: Facts) {
  return (
    <dl
      aria-label={t("agentFileTitle")}
      className="mt-3 rounded-[10px] bg-desk px-3.5 py-1 text-[12.5px]"
    >
      {agentRows(doc, body).map((row) => (
        <Fact key={keyOf(row)} row={row} onBackups={onBackups} onError={onError} />
      ))}
      {children}
    </dl>
  );
}

interface Backups {
  readonly place: Place;
  readonly restores: boolean;
  readonly onSee: (backup: number) => void;
  readonly onRestore: (backup: number) => void;
  readonly onClose: () => void;
}

export function BackupsModal({ place, restores, onSee, onRestore, onClose }: Backups) {
  return (
    <Modal title={t("agentBackupsAll")} onClose={onClose}>
      <p className="mb-2 text-[12.5px] text-soft">{fill("agentBackupsOf", place.at)}</p>
      <ul className="divide-y divide-hair text-[13px]">
        {place.kept.map((one) => (
          <li key={one.n} className="flex items-center justify-between gap-3 py-1.5">
            <span>{fill("agentBackupNamed", String(one.n), weigh(one.bytes))}</span>
            <span className="flex gap-2">
              <Link label={t("agentSee")} onClick={() => onSee(one.n)} />
              {restores && <Link label={t("agentRestore")} onClick={() => onRestore(one.n)} />}
            </span>
          </li>
        ))}
      </ul>
      <div className="mt-3 flex justify-end">
        <Action glyph="close" label={t("close")} onClick={onClose} />
      </div>
    </Modal>
  );
}

interface File {
  readonly shown: Shown;
  readonly restores: boolean;
  readonly onRestore: (backup: number) => void;
  readonly onClose: () => void;
}

export function FileModal({ shown, restores, onRestore, onClose }: File) {
  const backup = shown.backup;
  return (
    <Modal
      wide
      title={backup === null ? t("agentFileNow") : fill("agentBackupTitle", String(backup))}
      onClose={onClose}
    >
      <div className="flex h-[60vh] min-h-0 flex-col">
        <Page
          body={fencedFile(shown.text, shown.place.at)}
          label={shown.place.at}
          onDoc={() => undefined}
        />
      </div>
      <div className="mt-3 flex justify-end gap-2">
        {backup !== null && restores && (
          <Action glyph="history" label={t("agentRestore")} onClick={() => onRestore(backup)} />
        )}
        <Action glyph="close" label={t("close")} onClick={onClose} />
      </div>
    </Modal>
  );
}
