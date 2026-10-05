import { fill, t } from "../locales";

interface Props {
  name?: string;
  busy: boolean;
  onStay: () => void;
  onOther: () => void;
}

export default function Older({ name, busy, onStay, onOther }: Props) {
  return (
    <div className="flex flex-col gap-3">
      <div
        role="alert"
        className="rounded-[10px] border border-hue-amber/40 px-3 py-2 text-[11.5px] leading-relaxed text-soft"
      >
        <span className="block text-[12.5px] font-semibold text-ink">{t("welcomeOlder")}</span>
        {name ? fill("welcomeOlderBy", name) : t("welcomeOlderWhy")}
      </div>
      <div className="flex items-center justify-end gap-3">
        <button
          type="button"
          disabled={busy}
          onClick={onOther}
          className="text-[12.5px] text-faint hover:text-ink disabled:opacity-60"
        >
          {t("welcomeOtherFolder")}
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={onStay}
          className="rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60"
        >
          {t("welcomeStayHere")}
        </button>
      </div>
    </div>
  );
}
