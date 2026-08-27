import { useAsked } from "../asked";
import { taskLeft } from "../core";
import { weigh } from "../format";
import { t } from "../locales";

interface Props {
  task: string;
  heading?: React.ReactNode;
  onDoc?: (id: string) => void;
  onError?: (problem: unknown) => void;
}

const GLYPH = { doc: "▤", file: "📎", link: "↗", named: "◈" } as const;

export default function Left({ task, heading, onDoc, onError }: Props) {
  const left = useAsked(() => taskLeft(task), [task], onError);

  if (!left?.length) return null;

  return (
    <>
      {heading}
      <ul className="flex flex-col gap-1.5">
        {left.map((one) => {
          const doc = one.kind === "doc" ? one.target.replace("tisty:doc/", "") : null;
          const open = doc && !one.gone && onDoc ? () => onDoc(doc) : undefined;
          return (
            <li key={`${one.kind}-${one.target}`}>
              <button
                type="button"
                disabled={!open}
                onClick={open}
                className={`grid w-full grid-cols-[17px_minmax(0,1fr)_auto] items-baseline gap-2.5 rounded-lg border px-2.5 py-2 text-left ${
                  one.gone ? "border-dashed border-hair" : "border-hair"
                } ${open ? "cursor-pointer hover:border-line" : "cursor-default"}`}
              >
                <span aria-hidden="true" className="text-center text-xs text-faint">
                  {one.away ? "▢" : GLYPH[one.kind]}
                </span>
                <span className={`truncate text-[12.5px] ${one.gone ? "text-faint" : "text-ink"}`}>
                  {named(one)}
                </span>
                <span className="text-[11px] whitespace-nowrap text-faint">{said(one)}</span>
              </button>
            </li>
          );
        })}
      </ul>
    </>
  );
}

type One = NonNullable<ReturnType<typeof useAsked<Awaited<ReturnType<typeof taskLeft>>>>>[number];

function named(one: One): string {
  if (one.label) return one.label;
  if (one.kind === "link") {
    try {
      const at = new URL(one.target);
      return `${at.host}${at.pathname === "/" ? "" : at.pathname}`;
    } catch {
      return one.target;
    }
  }
  if (one.kind === "file") return one.target.split("/").pop() ?? one.target;
  return one.target;
}

function said(one: One): string {
  if (one.gone) return t("leftGone");
  if (one.away) return t("leftAway");
  if (one.kind === "file") return one.bytes === undefined ? "" : weigh(one.bytes);
  if (one.kind === "link") return t("leftLink");
  return "";
}
