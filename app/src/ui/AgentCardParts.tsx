import { fencedFile, type Place } from "../agentDocs";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
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

function Link({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button type="button" className="text-accent hover:underline" onClick={onClick}>
      {label}
    </button>
  );
}

interface Row {
  readonly place: Place;
  readonly code: boolean;
  readonly onBackups: () => void;
  readonly onTurn?: () => void;
}

export function PlaceRow({ place, code, onBackups, onTurn }: Row) {
  return (
    <div className="border-b border-hair py-2 text-[12.5px] last:border-b-0">
      <div className="flex items-center justify-between gap-3">
        <span className="flex shrink-0 items-center gap-1.5 text-soft">
          <Glyph name="upload" className="h-[13px] w-[13px]" />
          {fill("agentWrittenTo", place.named)}
        </span>
        <code className="min-w-0 truncate text-[11.5px]" title={place.at}>
          {place.at}
        </code>
      </div>
      <div className="mt-1 flex flex-wrap items-center justify-between gap-2 text-faint">
        <span>
          {t(code ? "agentAsCode" : "agentAsMarkdown")}
          {place.shared === 2 && ` · ${t("agentComposedOne")}`}
          {place.shared > 2 && ` · ${fill("agentComposed", String(place.shared - 1))}`}
          {place.changed && ` · ${fill("agentChangedAt", stamped(place.changed))}`}
        </span>
        <span className="flex items-center gap-2">
          {place.kept.length > 0 && (
            <Link
              label={
                place.kept.length === 1
                  ? t("agentBackupsOne")
                  : fill("agentBackups", String(place.kept.length))
              }
              onClick={onBackups}
            />
          )}
          {onTurn && (
            <Link
              label={fill(place.said.state === "off" ? "agentResume" : "agentPause", place.named)}
              onClick={onTurn}
            />
          )}
        </span>
      </div>
    </div>
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
