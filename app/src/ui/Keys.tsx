import type { Machine } from "../core";
import { fill, t } from "../locales";
import Modal from "./Modal";

export const briefly = (key: string): string => `${key.slice(0, 8)} … ${key.slice(-8)}`;

const grouped = (key: string): string[] => {
  const parts = key.match(/.{1,8}/g) ?? [];
  return [parts.slice(0, 4).join("  "), parts.slice(4).join("  ")].filter((one) => one !== "");
};

export const astrayKey = (one: Machine): boolean =>
  Boolean(one.confirmed) && Boolean(one.signs) && one.confirmed !== one.signs;

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
      title={one.mine ? t("theMachines") : fill("machineKeyTitle", one.called)}
      wide
      onClose={onClose}
    >
      <p className="text-[12.5px] leading-relaxed text-soft">
        {one.mine ? t("machineKeyMine") : fill("machineKeyRead", one.called)}
      </p>
      <div className="mt-3 rounded-[10px] border border-hair bg-panel px-3.5 py-3">
        {grouped(one.signs ?? "").map((row) => (
          <span
            key={row}
            className="block font-mono text-[13px] leading-[1.9] tracking-[0.04em] break-all whitespace-pre"
          >
            {row}
          </span>
        ))}
      </div>
      {!one.mine && (
        <>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
            {fill("machineKeyThen", one.called)}
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
  when: string;
  onClose: () => void;
}) {
  return (
    <Modal title={t("machineKeyAstray")} wide onClose={onClose}>
      <p className="text-[13px] font-semibold">{one.called}</p>
      <div className="mt-3 flex flex-wrap gap-7 text-[12.5px]">
        <span>
          <span className="block text-soft">{fill("machineAstrayStood", when)}</span>
          <span className="mt-0.5 block font-mono text-[12.5px] break-all">
            {briefly(one.confirmed ?? "")}
          </span>
        </span>
        <span>
          <span className="block text-soft">{t("machineAstrayNow")}</span>
          <span className="mt-0.5 block font-mono text-[12.5px] break-all text-urgent">
            {briefly(one.signs ?? "")}
          </span>
        </span>
      </div>
      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("machineAstrayWhat")}</p>
      <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("machineAstrayDo")}</p>
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

export function MachineRow({
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
  stood: string;
  onKey: (one: Machine) => void;
  onAstray: (one: Machine) => void;
  onDrop: (one: Machine) => void;
}) {
  return (
    <li
      className={`flex items-center justify-between gap-3 rounded-[10px] px-2.5 py-2 ${
        one.mine ? "bg-accent-soft" : ""
      }`}
    >
      <span className="min-w-0">
        <span className="block text-[13px] font-semibold">
          {one.called}
          {one.mine && (
            <span className="ml-2 rounded-full border border-accent px-1.5 py-px align-[1px] text-[10.5px] font-semibold tracking-wide text-accent uppercase">
              {t("machineHere")}
            </span>
          )}
        </span>
        <span className={`block text-[12.5px] ${quiet ? "text-ink" : "text-soft"}`}>{wrote}</span>
        <span className="block font-mono text-[10.5px] break-all text-faint">{one.id}</span>
        {!one.signs ? (
          <span className="mt-0.5 block text-[11.5px] text-faint">{t("machineKeyNone")}</span>
        ) : astrayKey(one) ? (
          <span className="mt-0.5 block text-[11.5px] font-semibold text-urgent">
            {t("machineKeyAstray")}
          </span>
        ) : (
          <span className="mt-0.5 block text-[11.5px] text-soft">
            {t("machineSigns")} <span className="font-mono text-ink">{briefly(one.signs)}</span>{" "}
            {one.confirmed ? (
              <span className="font-semibold text-hue-teal">{fill("machineKeyStands", stood)}</span>
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
        ) : one.signs && !one.confirmed && !one.mine ? (
          <button
            type="button"
            disabled={busy}
            onClick={() => onKey(one)}
            className="rounded-md border border-accent px-2.5 py-0.5 text-[12.5px] text-accent disabled:border-hair disabled:text-faint"
          >
            {t("machineKeyConfirm")}
          </button>
        ) : (
          one.signs && (
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
