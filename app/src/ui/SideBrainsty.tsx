import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  readonly onOpen: () => void;
  readonly rail?: boolean;
}

export default function SideBrainsty({ onOpen, rail = false }: Props) {
  if (rail) {
    return (
      <button
        type="button"
        onClick={onOpen}
        aria-label={t("brainsty")}
        title={t("brainstyTip")}
        className="grid size-9 shrink-0 place-items-center rounded-[10px] text-soft transition-colors duration-150 hover:bg-hover hover:text-ink"
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
        title={t("brainstyTip")}
        className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px] font-semibold text-faint transition-colors duration-150 hover:bg-hover hover:text-ink"
      >
        <Glyph name="network" />
        {t("brainsty")}
      </button>
    </div>
  );
}
