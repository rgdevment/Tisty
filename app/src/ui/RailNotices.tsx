import { fold } from "../breadth";
import { useAside, useComing, useStuck } from "../coming";
import { useKnocking } from "../knocking";
import { t } from "../locales";
import { useMove } from "../relocation";
import Knocking from "./Knocking";

export default function RailNotices() {
  const knocking = useKnocking();
  const move = useMove();
  const coming = useComing();
  const stuck = useStuck();
  const aside = useAside();
  const waiting =
    (knocking.waiting.length > 0 && !knocking.quiet) ||
    (move !== null && !move.watched) ||
    coming !== null ||
    Boolean(stuck) ||
    aside !== null;

  return (
    <>
      {knocking.asking && (
        <div className="h-0 overflow-hidden">
          <Knocking />
        </div>
      )}
      {waiting && (
        <button
          type="button"
          onClick={() => fold(false)}
          aria-label={t("sideNotices")}
          title={t("sideNotices")}
          className="grid size-7 place-items-center rounded-[10px] text-hue-amber transition-colors duration-150 hover:bg-hover"
        >
          <span
            aria-hidden="true"
            className="size-2 rounded-full bg-current motion-safe:animate-pulse"
          />
        </button>
      )}
    </>
  );
}
