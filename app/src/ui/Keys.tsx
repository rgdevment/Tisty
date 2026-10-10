import type { Machine } from "../core";
import { askAbout, shownAs } from "../knocking";
import { fill, t } from "../locales";
import Modal from "./Modal";

const HUSHED = 7 * 24 * 60 * 60;

export const hushed = (one: Machine): boolean =>
  !one.mine && !waiting(one) && (one.when === 0 || Date.now() / 1000 - one.when > HUSHED);

export const hushedName = (all: Machine[]): string | null => {
  const one = all.find(hushed);
  return one ? named(one) : null;
};

export const briefly = (key: string): string => `${key.slice(0, 8)} … ${key.slice(-8)}`;

// Two lines of four groups, so a person can read 64 characters out loud and not lose their place.
// Rendered as one text node with `whitespace-pre-line`: the rows have no identity of their own.
const grouped = (key: string): string => {
  const parts = key.match(/.{1,8}/g) ?? [];
  return [parts.slice(0, 4).join(" "), parts.slice(4).join(" ")]
    .filter((one) => one !== "")
    .join("\n");
};

// The round's own verdict first. Comparing the two keys only ever catches a store swapped under
// the file, because the log keeps the first key a machine published and never another.
export const astrayKey = (one: Machine): boolean =>
  one.turnedAway === "disowned" ||
  (Boolean(one.confirmed) && Boolean(one.signs) && one.confirmed !== one.signs);

export const waiting = (one: Machine): boolean => one.turnedAway === "unconfirmed";

const standing = (one: Machine): string | null => one.confirmed ?? one.signs ?? null;

export const named = shownAs;

// The code is what both windows show; the raw key stays for a machine that has not said one.
const spoken = (one: Machine): string => one.code ?? briefly(standing(one) ?? "");

const dated = (when: number): string | null => {
  if (when === 0) return null;
  const at = new Date(when * 1000);
  return Number.isNaN(at.getTime()) ? null : at.toLocaleDateString();
};

export function Asked({
  keyOf,
  astray,
  busy,
  onConfirm,
  onClose,
}: {
  keyOf: Machine | null;
  astray: Machine | null;
  busy: boolean;
  onConfirm: (one: Machine) => void;
  onClose: () => void;
}) {
  if (keyOf) {
    return <KeyOf one={keyOf} busy={busy} onConfirm={onConfirm} onClose={onClose} />;
  }
  if (astray) {
    return <KeyAstray one={astray} when={dated(astray.confirmedWhen)} onClose={onClose} />;
  }
  return null;
}

export function KeyOf({
  one,
  busy,
  onConfirm,
  onClose,
}: {
  one: Machine;
  busy: boolean;
  onConfirm: (one: Machine) => void;
  onClose: () => void;
}) {
  return (
    <Modal
      title={one.mine ? t("theMachines") : fill("machineKeyTitle", named(one))}
      wide
      onClose={onClose}
    >
      <p className="text-[12.5px] leading-relaxed text-soft">
        {one.mine ? t("machineKeyMine") : fill("machineKeyRead", named(one))}
      </p>
      <div className="mt-3 rounded-[10px] border border-hair bg-panel px-3.5 py-3">
        <span className="block font-mono text-[13px] leading-[1.9] tracking-[0.04em] break-all whitespace-pre-line">
          {one.code ?? grouped(standing(one) ?? "")}
        </span>
      </div>
      {!one.mine && (
        <>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
            {fill("machineKeyThen", named(one))}
          </p>
          <p className="mt-2 text-[12.5px] leading-relaxed text-hue-amber">
            {t("machineKeyForGood")}
          </p>
        </>
      )}
      <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
        <button
          type="button"
          onClick={onClose}
          className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
        >
          {one.mine ? t("done") : t("cancel")}
        </button>
        {!one.mine && (
          <button
            type="button"
            disabled={busy}
            onClick={() => onConfirm(one)}
            className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
          >
            {t("machineKeyConfirm")}
          </button>
        )}
      </div>
    </Modal>
  );
}

export function KeyAstray({
  one,
  when,
  onClose,
}: {
  one: Machine;
  when: string | null;
  onClose: () => void;
}) {
  return (
    <Modal title={t("machineKeyAstray")} wide onClose={onClose}>
      <p className="text-[13px] font-semibold">{named(one)}</p>
      <div className="mt-3 flex flex-wrap gap-7 text-[12.5px]">
        <span>
          <span className="block text-soft">
            {when === null ? t("machineAstrayStoodEver") : fill("machineAstrayStood", when)}
          </span>
          <span className="mt-0.5 block font-mono text-[12.5px] tracking-[0.04em] break-all whitespace-pre-line">
            {grouped(one.confirmed ?? "")}
          </span>
        </span>
        <span>
          <span className="block text-soft">{t("machineAstrayNow")}</span>
          <span className="mt-0.5 block font-mono text-[12.5px] tracking-[0.04em] break-all whitespace-pre-line text-urgent">
            {grouped(one.signs ?? "")}
          </span>
        </span>
      </div>
      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("machineAstrayWhat")}</p>
      <p className="mt-2 text-[12.5px] leading-relaxed text-soft">
        {one.mine ? t("machineAstrayMine") : t("machineAstrayDo")}
      </p>
      <p className="mt-3 text-[12.5px] leading-relaxed text-faint">{t("machineAstrayOnly")}</p>
      <div className="mt-5 flex items-center justify-end text-[12.5px]">
        <button
          type="button"
          onClick={onClose}
          className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
        >
          {t("done")}
        </button>
      </div>
    </Modal>
  );
}

export function MachineList({
  all,
  busy,
  onKey,
  onAstray,
  onDrop,
}: {
  all: Machine[] | null;
  busy: boolean;
  onKey: (one: Machine) => void;
  onAstray: (one: Machine) => void;
  onDrop: (one: Machine) => void;
}) {
  if (all === null) {
    return null;
  }
  if (all.length === 0) {
    return <p className="mt-2 text-[12.5px] text-faint">{t("machinesNone")}</p>;
  }
  return (
    <>
      <ul className="mt-2 flex flex-col gap-1 text-[12.5px]">
        {all.map((one) => (
          <MachineRow
            key={one.id}
            one={one}
            busy={busy}
            quiet={hushed(one)}
            wrote={
              one.when === 0
                ? one.since
                  ? fill("machineSince", dated(one.since) ?? "")
                  : t("machineNever")
                : (dated(one.when) ?? "")
            }
            stood={dated(one.confirmedWhen)}
            onKey={onKey}
            onAstray={onAstray}
            onDrop={onDrop}
          />
        ))}
      </ul>
      {all.some(hushed) && (
        <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("machineHushed")}</p>
      )}
    </>
  );
}

function MachineRow({
  one,
  busy,
  quiet,
  wrote,
  stood,
  onKey,
  onAstray,
  onDrop,
}: {
  one: Machine;
  busy: boolean;
  quiet: boolean;
  wrote: string;
  stood: string | null;
  onKey: (one: Machine) => void;
  onAstray: (one: Machine) => void;
  onDrop: (one: Machine) => void;
}) {
  return (
    <li
      className={`flex items-center justify-between gap-3 rounded-[10px] px-2.5 py-2 ${
        waiting(one)
          ? "border border-hue-amber/40 bg-hue-amber/10"
          : one.mine
            ? "bg-accent-soft"
            : ""
      }`}
    >
      <span className="min-w-0">
        <span className="block text-[13px] font-semibold">
          {named(one)}
          {one.mine && (
            <span className="ml-2 rounded-full border border-accent px-1.5 py-px align-[1px] text-[10.5px] font-semibold tracking-wide text-accent uppercase">
              {t("machineHere")}
            </span>
          )}
          {waiting(one) && (
            <span className="ml-2 rounded-full border border-hue-amber px-1.5 py-px align-[1px] text-[10.5px] font-semibold tracking-wide text-hue-amber uppercase">
              {t("machineWaits")}
            </span>
          )}
        </span>
        <span className={`block text-[12.5px] ${quiet ? "text-ink" : "text-soft"}`}>
          {[one.os, wrote].filter(Boolean).join(" · ")}
        </span>
        <span className="block font-mono text-[10.5px] break-all text-faint">{one.id}</span>
        {astrayKey(one) ? (
          <span className="mt-0.5 block text-[11.5px] font-semibold text-urgent">
            {t("machineKeyAstray")}
          </span>
        ) : waiting(one) ? (
          <>
            {standing(one) === null ? (
              <span className="mt-0.5 block text-[11.5px] text-faint">{t("machineKeyNone")}</span>
            ) : (
              <span className="mt-0.5 block text-[11.5px] text-soft">
                {t("machineCode")} <span className="font-mono text-ink">{spoken(one)}</span>
              </span>
            )}
            <span className="mt-0.5 block text-[11.5px] font-semibold text-hue-amber">
              {t("machineWaitsWhy")}
            </span>
          </>
        ) : standing(one) === null ? (
          <span className="mt-0.5 block text-[11.5px] text-faint">{t("machineKeyNone")}</span>
        ) : (
          <span className="mt-0.5 block text-[11.5px] text-soft">
            {t("machineCode")} <span className="font-mono text-ink">{spoken(one)}</span>{" "}
            {one.mine ? (
              <span className="text-faint">{t("machineKeyOurs")}</span>
            ) : one.carried ? (
              <span className="text-soft">{t("machineCarried")}</span>
            ) : one.rotated ? (
              <span className="text-soft">{t("machineRotated")}</span>
            ) : one.confirmed ? (
              <span className="font-semibold text-hue-teal">
                {stood === null ? t("machineKeyStandsEver") : fill("machineKeyStands", stood)}
              </span>
            ) : (
              <span className="text-hue-amber">{t("machineKeyLoose")}</span>
            )}
          </span>
        )}
      </span>
      <span className="flex shrink-0 items-center gap-2.5">
        {astrayKey(one) ? (
          <button
            type="button"
            onClick={() => onAstray(one)}
            className="text-[12.5px] text-urgent underline decoration-urgent/40"
          >
            {t("machineKeyAsk")}
          </button>
        ) : standing(one) && !one.confirmed && !one.mine ? (
          <button
            type="button"
            disabled={busy}
            onClick={() => (waiting(one) ? askAbout(one) : onKey(one))}
            className="rounded-md border border-accent px-2.5 py-0.5 text-[12.5px] text-accent disabled:border-hair disabled:text-faint"
          >
            {t("machineKeyConfirm")}
          </button>
        ) : (
          standing(one) && (
            <button
              type="button"
              onClick={() => onKey(one)}
              className="text-[12.5px] text-accent underline decoration-accent/40"
            >
              {t("machineKeySee")}
            </button>
          )
        )}
        {one.mine ? (
          <span className="text-[12.5px] text-faint">{t("machineNeverDrop")}</span>
        ) : (
          <button
            type="button"
            disabled={busy}
            onClick={() => onDrop(one)}
            className="rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-urgent hover:text-urgent disabled:text-faint"
          >
            {t("machineDrop")}
          </button>
        )}
      </span>
    </li>
  );
}
