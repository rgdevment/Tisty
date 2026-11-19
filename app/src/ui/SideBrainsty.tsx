import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  readonly onOpen: () => void;
  readonly on?: boolean;
  readonly rail?: boolean;
}

export default function SideBrainsty({ onOpen, on = false, rail = false }: Props) {
  if (rail) {
    return (
      <button
        type="button"
        onClick={onOpen}
        aria-label={t("brainsty")}
        aria-current={on ? "page" : undefined}
        title={t("brainstyTip")}
        className={`grid size-9 shrink-0 place-items-center rounded-[10px] transition-colors duration-150 hover:bg-hover ${
          on ? "bg-active text-accent" : "text-soft hover:text-ink"
        }`}
      >
        <Glyph name="network" />
      </button>
    );
  }
  return (
    <div className="shrink-0 px-2.5 pt-1.5">
      <button
        type="button"
        onClick={onOpen}
        aria-current={on ? "page" : undefined}
        title={t("brainstyTip")}
        className={`flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px] font-semibold transition-colors duration-150 hover:bg-hover hover:text-ink ${
          on ? "bg-active text-ink" : "text-faint"
        }`}
      >
        <Glyph name="network" />
        {t("brainsty")}
      </button>
    </div>
  );
}
