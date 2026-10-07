import type { Afoot as Going, Ready, Underway } from "../core";
import { fill, t, type Word } from "../locales";

interface Alarming {
  error: string;
  stuck: boolean;
  offer: Ready | null;
  underway: Underway | null;
  onTakeMe: () => void;
  onInstall: () => void;
  onClose: () => void;
}

export function Alarm({ error, stuck, offer, underway, onTakeMe, onInstall, onClose }: Alarming) {
  return (
    <div
      role="alert"
      className="shadow-lift fixed inset-x-0 top-11 z-[60] mx-auto flex w-fit max-w-[70%] items-start gap-2.5 rounded-[10px] border border-urgent/40 bg-bg px-3.5 py-2 text-[12.5px] leading-snug text-urgent"
    >
      <span className="select-text">{error}</span>
      {stuck && (
        <button
          type="button"
          onClick={onTakeMe}
          className="shrink-0 rounded-md border border-urgent/40 px-1.5 py-0.5 hover:bg-urgent/10"
        >
          {t("stuckTakeMe")}
        </button>
      )}
      {offer?.installs && (
        <button
          type="button"
          disabled={!!underway}
          onClick={onInstall}
          className="shrink-0 rounded-md border border-urgent/40 px-1.5 py-0.5 hover:bg-urgent/10"
        >
          {t(
            underway
              ? offer.route === "store"
                ? "updateInstallingStore"
                : "updateInstalling"
              : "updateInstall",
          )}
        </button>
      )}
      <button
        type="button"
        aria-label={t("close")}
        onClick={onClose}
        className="-mr-1 shrink-0 rounded-md px-1 hover:bg-urgent/10"
      >
        ✕
      </button>
    </div>
  );
}

export function Progress({
  settling,
  note,
  afoot,
}: {
  settling: boolean;
  note: string | null;
  afoot: Going | null;
}) {
  return (
    <>
      {settling && (
        <p className="pointer-events-none fixed inset-x-0 top-11 z-[60] mx-auto w-fit rounded-md bg-accent-soft px-3 py-1.5 text-[11.5px] text-accent">
          {t("settlingIn")}
        </p>
      )}

      {note && (
        <p
          role="status"
          className="pointer-events-none fixed bottom-5 left-1/2 z-[60] w-fit -translate-x-1/2 rounded-[10px] border border-hair bg-rail px-3.5 py-2 text-[11.5px] text-ink shadow-xl"
        >
          {note}
        </p>
      )}

      {afoot && (
        <p
          role="status"
          aria-live="polite"
          className="pointer-events-none fixed bottom-5 left-1/2 z-[60] w-64 -translate-x-1/2 rounded-[10px] border border-hair bg-rail px-3.5 py-2 text-[11.5px] text-ink shadow-xl"
        >
          <span className="block">
            {fill(`${afoot.stage}On` as Word, afoot.far ? `${afoot.far} %` : "").trim()}
          </span>
          <span className="mt-0.5 block text-[11.5px] text-soft">{t("aWhileYet")}</span>
          <span className="mt-1.5 block h-1 overflow-hidden rounded-full bg-desk">
            <span
              className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
              style={{ width: `${afoot.far}%` }}
            />
          </span>
        </p>
      )}
    </>
  );
}
